mod dropdowns;
mod factory;
pub(crate) mod icons;
mod methods;
mod modules;
mod styling;
mod watchers;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use factory::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk, prelude::*};
use wayle_config::{ConfigProperty, schemas::bar::BarLayout};
use wayle_widgets::{prelude::BarSettings, styling::InlineStyling};

use self::dropdowns::{DropdownOpener, DropdownRegistry};
use crate::{
    services::shell_ipc::DropdownAction,
    shell::{helpers::layer_shell::apply_layer, services::ShellServices},
};

pub(crate) struct Bar {
    settings: BarSettings,
    services: ShellServices,
    dropdowns: Rc<DropdownRegistry>,
    layout: BarLayout,
    css_provider: gtk::CssProvider,
    last_css: String,

    left: FactoryVecDeque<BarItemFactory>,
    center: FactoryVecDeque<BarItemFactory>,
    right: FactoryVecDeque<BarItemFactory>,

    /// Maps a dropdown identifier (e.g. `audio@microphone`) to the module's opener
    /// and the dropdown name it toggles, rebuilt from the live modules whenever the
    /// layout changes. Backs `wayle dropdown toggle` — dispatching through the
    /// same opener the module's own click uses.
    dropdown_targets: RefCell<HashMap<String, (DropdownOpener, String)>>,
}

pub(crate) struct BarInit {
    pub(crate) monitor: gdk::Monitor,
    pub(crate) services: ShellServices,
}

#[derive(Debug)]
pub(crate) enum BarCmd {
    LayoutLoaded(BarLayout),
    StyleChanged,
    ExclusiveChanged(bool),
    LayerChanged,
    /// A CLI dropdown request (`toggle`/`open`/`close`) targeting this bar. The
    /// `String` is the dropdown identifier (empty for [`DropdownAction::Close`]).
    Dropdown(DropdownAction, String),
    /// The config was reloaded; re-derive the dropdown identifier map and republish
    /// it (openers read their names live, so a re-bound click updates `dropdown list`).
    RepublishDropdowns,
}

#[relm4::component(pub(crate))]
impl Component for Bar {
    type Init = BarInit;
    type Input = ();
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

        let visible_on_startup =
            Self::visible_on_startup(config, &ipc_state, monitor_name.as_deref().unwrap_or("unknown"));

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

        root.init_layer_shell();
        apply_layer(&root, config.bar.layer.get(), &init.services.config);
        root.set_keyboard_mode(KeyboardMode::None);
        Self::apply_exclusive_zone(&root, config.bar.exclusive.get());
        root.set_monitor(Some(&init.monitor));
        Self::apply_anchors(&root, location);
        Self::apply_css_classes(&root, &init.monitor, location, is_floating);
        Self::suppress_alt_focus(&root);

        let window = root.clone();
        init.monitor.connect_invalidate(move |_| {
            window.destroy();
        });

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

        watchers::layout::spawn(&sender, &init.monitor, &init.services.config, &ipc_state);
        watchers::exclusive::spawn(&sender, &init.services.config);
        watchers::layer::spawn(&sender, &init.services.config);
        watchers::dropdown::spawn(&sender, &init.monitor, &init.services.config, &ipc_state);

        let dropdowns = Rc::new(DropdownRegistry::new(&init.services, &init.monitor, &root));
        dropdowns.warm_all();
        dropdowns.set_republish({
            let command = sender.command_sender().clone();
            Rc::new(move || {
                let _ = command.send(BarCmd::RepublishDropdowns);
            })
        });

        Self::install_dismiss_controllers(&root, &dropdowns);

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
            left,
            center,
            right,
            dropdown_targets: RefCell::new(HashMap::new()),
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

    fn update_cmd(&mut self, msg: BarCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            BarCmd::LayoutLoaded(layout) => {
                self.apply_layout(layout, root);
                self.rebuild_dropdown_targets();
            }
            BarCmd::StyleChanged => {
                let new_css = self.build_css();
                if new_css != self.last_css {
                    self.css_provider.load_from_string(&new_css);
                    self.last_css = new_css;
                }
            }
            BarCmd::ExclusiveChanged(exclusive) => {
                Self::apply_exclusive_zone(root, exclusive);
            }
            BarCmd::LayerChanged => {
                // While a dropdown/menu is open the scrim has raised the bar to Overlay
                // and owns its layer; re-layering now would drop it below the active
                // full-monitor scrim (which would then swallow the popover's clicks).
                // Defer — `Scrim::hide` re-reads and applies the configured layer when
                // the surface closes.
                if !self.dropdowns.coordinator().has_open_surface() {
                    let configured = self.services.config.config().bar.layer.get();
                    apply_layer(root, configured, &self.services.config);
                }
            }
            BarCmd::Dropdown(action, identifier) => {
                self.handle_dropdown_request(action, &identifier, root);
            }
            BarCmd::RepublishDropdowns => {
                self.rebuild_dropdown_targets();
            }
        }
    }
}
