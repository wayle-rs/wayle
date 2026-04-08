mod dropdowns;
mod factory;
pub(crate) mod icons;
mod methods;
mod modules;
pub(crate) mod pomodoro;
mod styling;
mod watchers;

use std::rc::Rc;

use factory::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, Layer, LayerShell};
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk, prelude::*};
use wayle_config::{ConfigProperty, schemas::bar::BarLayout};
use wayle_widgets::{prelude::BarSettings, styling::InlineStyling};

use self::dropdowns::DropdownRegistry;
use crate::shell::services::ShellServices;

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
}

pub(crate) struct BarInit {
    pub(crate) monitor: gdk::Monitor,
    pub(crate) services: ShellServices,
}

#[derive(Debug)]
pub(crate) enum BarCmd {
    LayoutLoaded(BarLayout),
    StyleChanged,
    DropdownAutohideChanged(bool),
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

        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_keyboard_mode(KeyboardMode::None);
        root.auto_exclusive_zone_enable();
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
        watchers::dropdowns::spawn(&sender, &init.services.config);

        let dropdowns = Rc::new(DropdownRegistry::new(&init.services));
        dropdowns.warm_all();

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
        }
    }
}
