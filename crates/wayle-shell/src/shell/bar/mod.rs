mod autohide;
mod dropdowns;
mod factory;
pub(crate) mod icons;
mod methods;
mod modules;
mod styling;
mod watchers;

use std::rc::Rc;

use factory::*;
use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::{bar::BarLayout, styling::Spacing},
};
use wayle_widgets::{prelude::BarSettings, styling::InlineStyling};

use self::{
    autohide::{AutohideState, HoverTrigger},
    dropdowns::DropdownRegistry,
};
use crate::shell::{helpers::layer_shell::apply_layer, services::ShellServices};

pub(crate) struct Bar {
    settings: BarSettings,
    services: ShellServices,
    dropdowns: Rc<DropdownRegistry>,
    layout: BarLayout,
    css_provider: gtk::CssProvider,
    last_css: String,
    monitor: gdk::Monitor,
    autohide_state: AutohideState,
    hover_trigger: Option<HoverTrigger>,

    left: FactoryVecDeque<BarItemFactory>,
    center: FactoryVecDeque<BarItemFactory>,
    right: FactoryVecDeque<BarItemFactory>,
}

pub(crate) struct BarInit {
    pub(crate) monitor: gdk::Monitor,
    pub(crate) services: ShellServices,
}

/// User-driven inputs handled by the [`Bar`] component itself (as opposed to
/// [`BarCmd`], which carries background-task/config-watcher outputs).
#[derive(Debug)]
pub(crate) enum BarInput {
    /// Pointer entered the bar's own surface.
    HoverEnter,
    /// Pointer moved while inside the bar's surface.
    HoverMotion,
    /// Pointer left the bar's surface.
    HoverLeave,
    /// Pointer entered the separate `HoverTrigger` edge-strip surface (not
    /// the bar's own surface). Handled distinctly from `HoverEnter` because
    /// the trigger unmaps as soon as the bar reveals, so it can never
    /// guarantee a matching leave event -- see
    /// `AutohideState::on_trigger_enter`.
    TriggerHover,
    /// A dropdown popover opened. Autohide relies on this to stay revealed
    /// while a dropdown is open, even if it was opened through a path that
    /// didn't itself hover the bar.
    DropdownOpened,
    /// A dropdown popover closed. Autohide relies on this to resume its
    /// hide timer when a dropdown closes without the pointer being back
    /// over the bar (e.g. clicking a menu item, or an outside click).
    DropdownClosed,
}

#[derive(Debug)]
pub(crate) enum BarCmd {
    LayoutLoaded(BarLayout),
    StyleChanged,
    DropdownAutohideChanged(bool),
    ExclusiveChanged(bool),
    LayerChanged,
    AutohideTimeout(u64),
    AutohideChanged(bool),
    AutohideTimeoutChanged(u32),
    AutohideTriggerSizeChanged(Spacing),
}

#[relm4::component(pub(crate))]
impl Component for Bar {
    type Init = BarInit;
    type Input = BarInput;
    type Output = ();
    type CommandOutput = BarCmd;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
            add_css_class: "bar",
            set_size_request: (1, 1),

            #[name = "center_box"]
            gtk::CenterBox {
                #[wrap(Some)]
                #[name = "left_box"]
                set_start_widget = &gtk::Box {
                    add_css_class: "bar-section",
                    add_css_class: "bar-left",
                },

                #[wrap(Some)]
                #[name = "middle_box"]
                set_center_widget = &gtk::Box {
                    add_css_class: "bar-section",
                    add_css_class: "bar-center",
                },

                #[wrap(Some)]
                #[name = "right_box"]
                set_end_widget = &gtk::Box {
                    add_css_class: "bar-section",
                    add_css_class: "bar-right",
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.services.config.config();
        let location = config.bar.location.get();
        let inset_edge = config.bar.inset_edge.get().value();
        let inset_ends = config.bar.inset_ends.get().value();
        let is_floating = inset_edge > 0.0 || inset_ends > 0.0;

        let monitor_name = init.monitor.connector().map(|s| s.to_string());

        let ipc_state = init.services.shell_ipc.state();

        let visible_on_startup = {
            let connector = monitor_name.as_deref().unwrap_or("unknown");
            let layouts = config.bar.layout.get();
            let config_visible = watchers::layout::find_layout(&layouts, connector)
                .is_some_and(|layout| layout.show);

            config_visible && !ipc_state.hidden_bars.get().contains(connector)
        };

        let settings = BarSettings {
            variant: config.bar.button_variant.clone(),
            theme_provider: config.styling.theme_provider.clone(),
            border_location: config.bar.button_border_location.clone(),
            border_width: config.bar.button_border_width.clone(),
            icon_position: config.bar.button_icon_position.clone(),
            is_vertical: ConfigProperty::new(location.is_vertical()),
            scroll_sensitivity: 1.0,
            monitor_name,
        };

        Self::configure_root_window(
            &root,
            &init.monitor,
            &init.services.config,
            location,
            is_floating,
        );

        Self::attach_motion_controller(&root, &sender);

        let left = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let center = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let right = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let css_provider = gtk::CssProvider::new();

        #[allow(deprecated)]
        root.style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);

        Self::spawn_watchers(&sender, &init.monitor, &init.services.config, &ipc_state);

        let (autohide_state, hover_trigger) =
            Self::init_autohide(config, &init.monitor, location, &sender);

        let dropdowns = Self::init_dropdowns(&init.services, &sender);

        let mut model = Self {
            settings,
            services: init.services,
            dropdowns,
            layout: BarLayout {
                monitor: String::new(),
                extends: None,
                show: true,
                left: Vec::new(),
                center: Vec::new(),
                right: Vec::new(),
            },
            css_provider,
            last_css: String::new(),
            monitor: init.monitor,
            autohide_state,
            hover_trigger,
            left,
            center,
            right,
        };

        model.spawn_style_watcher(&sender);
        model.last_css = model.build_css();
        model.css_provider.load_from_string(&model.last_css);

        let widgets = view_output!();

        let is_vert = model.settings.is_vertical.get();
        Self::apply_orientations(
            &widgets.center_box,
            &widgets.left_box,
            &widgets.middle_box,
            &widgets.right_box,
            model.left.widget(),
            model.center.widget(),
            model.right.widget(),
            is_vert,
        );

        widgets.left_box.append(model.left.widget());
        widgets.middle_box.append(model.center.widget());
        widgets.right_box.append(model.right.widget());

        if visible_on_startup {
            root.present();
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: BarInput, sender: ComponentSender<Self>, root: &Self::Root) {
        // Autohide is off for most users (default `autohide = false`), and
        // hover motion over the bar fires at very high frequency. Skip the
        // state-machine dispatch entirely rather than paying for a mutation
        // + debug log + visibility resync on every pointer move when there's
        // nothing for autohide to do.
        if matches!(
            msg,
            BarInput::HoverEnter | BarInput::HoverMotion | BarInput::HoverLeave
        ) && !self.autohide_state.is_enabled()
        {
            return;
        }

        let action = match msg {
            BarInput::HoverEnter => self.autohide_state.on_hover_enter(),
            BarInput::HoverMotion => self.autohide_state.on_hover_motion(),
            BarInput::HoverLeave => {
                // Registry-tracked dropdowns stay fully event-driven:
                // `DropdownClosed` re-arms via `on_popover_closed` once one
                // closes, so it's safe to `CancelTimer`-and-wait here.
                // Popovers only visible to the generic tree walk (e.g. a
                // systray context menu) have no such close event, so they're
                // deliberately excluded from this bool: instead of an
                // indefinite `CancelTimer` with nothing left to wake it back
                // up, this arms a normal hide timer anyway, and
                // `BarCmd::AutohideTimeout` below re-checks fresh at expiry
                // and re-arms again if such a popover is still open.
                let registry_open = self.dropdowns.any_open();
                self.autohide_state.on_hover_leave(registry_open)
            }
            BarInput::TriggerHover => self.autohide_state.on_trigger_enter(),
            BarInput::DropdownOpened => self.autohide_state.on_popover_open(),
            BarInput::DropdownClosed => self.autohide_state.on_popover_closed(),
        };

        self.handle_autohide_action(action, &sender, root);
    }

    fn update_cmd(&mut self, msg: BarCmd, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            BarCmd::LayoutLoaded(layout) => {
                self.apply_layout(layout, root);
            }
            BarCmd::StyleChanged => {
                let new_css = self.build_css();
                if new_css != self.last_css {
                    self.css_provider.load_from_string(&new_css);
                    self.last_css = new_css;
                }
            }
            BarCmd::DropdownAutohideChanged(autohide) => {
                self.dropdowns.set_all_autohide(autohide);
            }
            BarCmd::ExclusiveChanged(exclusive) => {
                Self::apply_exclusive_zone(root, exclusive);
            }
            BarCmd::LayerChanged => {
                let configured = self.services.config.config().bar.layer.get();
                apply_layer(root, configured, &self.services.config);
            }
            BarCmd::AutohideTimeout(token) => {
                let registry_open = self.dropdowns.any_open();
                // Only worth walking the tree if the (cheap) registry check
                // didn't already answer it -- see
                // `AutohideState::on_autohide_timeout`'s doc comment for why
                // the two sources are kept distinct rather than merged here.
                let walk_open =
                    !registry_open && dropdowns::any_popover_open_in_tree(root.upcast_ref());

                let action =
                    self.autohide_state
                        .on_autohide_timeout(token, registry_open, walk_open);
                self.handle_autohide_action(action, &sender, root);
            }
            BarCmd::AutohideChanged(enabled) => {
                let action = self.autohide_state.set_enabled(enabled);
                self.sync_hover_trigger(enabled, &sender, root);
                self.handle_autohide_action(action, &sender, root);
            }
            BarCmd::AutohideTimeoutChanged(timeout_ms) => {
                self.autohide_state.set_timeout(timeout_ms);
            }
            BarCmd::AutohideTriggerSizeChanged(size) => {
                self.refresh_trigger_geometry(size);
            }
        }
    }
}
