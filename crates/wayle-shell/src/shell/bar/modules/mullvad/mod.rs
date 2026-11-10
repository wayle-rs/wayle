mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ClickAction, ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_core::DeferredService;
use wayle_mullvad::{ConnectionStatus, MullvadService};
use wayle_widgets::{
    WatcherToken,
    prelude::{BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonOutput},
};

pub(crate) use self::{
    factory::Factory,
    messages::{MullvadCmd, MullvadInit, MullvadMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct MullvadModule {
    bar_button: Controller<BarButton>,
    state_watcher: WatcherToken,
    mullvad: DeferredService<MullvadService>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for MullvadModule {
    type Init = MullvadInit;
    type Input = MullvadMsg;
    type Output = ();
    type CommandOutput = MullvadCmd;

    view! {
        gtk::Box {
            add_css_class: "mullvad",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let mullvad_config = &config.modules.mullvad;

        let (initial_icon, initial_label) =
            Self::compute_display(mullvad_config, &init.mullvad.get());

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: mullvad_config.icon_color.clone(),
                    label_color: mullvad_config.label_color.clone(),
                    icon_background: mullvad_config.icon_bg_color.clone(),
                    button_background: mullvad_config.button_bg_color.clone(),
                    border_color: mullvad_config.border_color.clone(),
                    auto_icon_color: CssToken::Green,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: mullvad_config.label_max_length.clone(),
                    show_icon: mullvad_config.icon_show.clone(),
                    show_label: mullvad_config.label_show.clone(),
                    show_border: mullvad_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => MullvadMsg::LeftClick,
                BarButtonOutput::RightClick => MullvadMsg::RightClick,
                BarButtonOutput::MiddleClick => MullvadMsg::MiddleClick,
                BarButtonOutput::ScrollUp => MullvadMsg::ScrollUp,
                BarButtonOutput::ScrollDown => MullvadMsg::ScrollDown,
            });

        watchers::spawn_service_watcher(&sender, &init.mullvad);
        watchers::spawn_icon_config_watcher(&sender, mullvad_config);
        let state_watcher = WatcherToken::new();

        let model = Self {
            bar_button,
            state_watcher,
            mullvad: init.mullvad,
            config: init.config,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.mullvad;

        let action = match msg {
            MullvadMsg::LeftClick => config.left_click.get(),
            MullvadMsg::RightClick => config.right_click.get(),
            MullvadMsg::MiddleClick => config.middle_click.get(),
            MullvadMsg::ScrollUp => config.scroll_up.get(),
            MullvadMsg::ScrollDown => config.scroll_down.get(),
        };

        // `:toggle` is a mullvad-specific action (default right-click); every
        // other action goes through the shared dispatcher (`None` = no-op).
        if matches!(&action, ClickAction::Shell(command) if command == ":toggle") {
            self.toggle();
        } else {
            dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
        }
    }

    fn update_cmd(&mut self, msg: MullvadCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        let mullvad_config = &self.config.config().modules.mullvad;

        match msg {
            MullvadCmd::ServiceReady(service) => {
                let token = self.state_watcher.reset();
                watchers::spawn_state_watchers(&sender, token, &service);
                self.update_display(mullvad_config, &Some(service));
            }
            MullvadCmd::StateChanged | MullvadCmd::IconConfigChanged => {
                self.update_display(mullvad_config, &self.mullvad.get());
            }
        }
    }
}

impl MullvadModule {
    /// Best-effort connect/disconnect toggle: connect to the daemon's selected
    /// relay when disconnected, disconnect otherwise, and do nothing without a
    /// logged-in account. The calls are queued by the service, so non-blocking.
    fn toggle(&self) {
        let Some(service) = self.mullvad.get() else {
            return;
        };

        match service.mullvad.status.get() {
            ConnectionStatus::Disconnected => service.mullvad.connect(),
            // Nothing to toggle without a logged-in account.
            ConnectionStatus::LoggedOut | ConnectionStatus::Revoked => {}
            _ => service.mullvad.disconnect(),
        }
    }
}
