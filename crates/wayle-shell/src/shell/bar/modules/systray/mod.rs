mod factory;
mod helpers;
mod item;
mod messages;
mod methods;
mod styling;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk4::prelude::{OrientableExt, WidgetExt};
use item::{SystrayItem, SystrayItemMsg};
use relm4::{ComponentParts, ComponentSender, factory::FactoryVecDeque, gtk, prelude::*};
use wayle_config::{ConfigProperty, ConfigService};
use wayle_widgets::prelude::{
    BarContainer, BarContainerBehavior, BarContainerColors, BarContainerInit, force_window_resize,
};

pub(crate) use self::{
    factory::Factory,
    messages::{SystrayCmd, SystrayInit, SystrayMsg},
};
use crate::{services::shell_ipc::SystrayMenuAction, shell::bar::dropdowns::OpenSurfaceCoordinator};

pub(crate) struct SystrayModule {
    container: Controller<BarContainer>,
    items: FactoryVecDeque<SystrayItem>,
    css_provider: gtk::CssProvider,
    visible: ConfigProperty<bool>,
    config: Arc<ConfigService>,
    coordinator: Rc<OpenSurfaceCoordinator>,
}

#[relm4::component(pub(crate))]
impl Component for SystrayModule {
    type Init = SystrayInit;
    type Input = SystrayMsg;
    type Output = ();
    type CommandOutput = SystrayCmd;

    view! {
        gtk::Box {
            add_css_class: "systray",

            #[local_ref]
            container -> gtk::Box {
                #[local_ref]
                items_box -> gtk::Box {},
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let full_config = init.config.config();
        let config = &full_config.modules.systray;
        let styling_config = &full_config.styling;
        let bar_config = &full_config.bar;

        let visible = ConfigProperty::new(false);

        let container = BarContainer::builder()
            .launch(BarContainerInit {
                colors: BarContainerColors {
                    background: config.button_bg_color.clone(),
                    border_color: config.border_color.clone(),
                },
                behavior: BarContainerBehavior {
                    show_border: config.border_show.clone(),
                    visible: visible.clone(),
                },
                is_vertical: init.is_vertical.clone(),
                theme_provider: styling_config.theme_provider.clone(),
                border_width: bar_config.button_border_width.clone(),
                border_location: bar_config.button_border_location.clone(),
            })
            .detach();

        let orientation = if init.is_vertical.get() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        };
        let items = FactoryVecDeque::builder()
            .launch(gtk::Box::new(orientation, 0))
            .detach();

        let css_provider = styling::init_css_provider(items.widget(), &init.config);

        watchers::spawn_watchers(
            &sender,
            &init.is_vertical,
            &init.systray,
            &init.config,
            &init.shell_ipc,
            init.monitor.clone(),
        );

        let model = Self {
            container,
            items,
            css_provider,
            visible,
            config: init.config,
            coordinator: init.coordinator,
        };
        let container = model.container.widget();
        let items_box = model.items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            SystrayCmd::ItemsChanged(items) => {
                self.update_items(items);
                if let Some(parent) = root.parent() {
                    parent.set_visible(self.visible.get());
                }
                force_window_resize(root);
            }
            SystrayCmd::StylingChanged => {
                styling::reload_css(&self.css_provider, &self.config);
                force_window_resize(root);
            }
            SystrayCmd::OrientationChanged(vertical) => {
                let orientation = if vertical {
                    gtk::Orientation::Vertical
                } else {
                    gtk::Orientation::Horizontal
                };
                self.items.widget().set_orientation(orientation);
                force_window_resize(root);
            }
            SystrayCmd::MenuRequest(action, id) => {
                // Route through the item's own menu path (the same one its
                // right-click uses), so CLI and mouse behave identically.
                let msg = match action {
                    SystrayMenuAction::Toggle => SystrayItemMsg::ToggleMenu,
                    SystrayMenuAction::Open => SystrayItemMsg::OpenMenu,
                };
                for index in 0..self.items.len() {
                    if self.items.get(index).is_some_and(|item| item.tray_id() == id) {
                        self.items.send(index, msg);
                        break;
                    }
                }
            }
        }
    }
}
