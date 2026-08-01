mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_network::NetworkService;
use wayle_widgets::{
    WatcherToken,
    prelude::{
        BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput,
        BarButtonOutput,
    },
};

pub(crate) use self::{
    factory::Factory,
    messages::{NetworkCmd, NetworkInit, NetworkMsg},
};
use crate::shell::bar::dropdowns::DropdownOpener;

pub(crate) struct NetworkModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    wifi_watcher: WatcherToken,
    wired_watcher: WatcherToken,
    network: Arc<NetworkService>,
    opener: DropdownOpener,
}

#[relm4::component(pub(crate))]
impl Component for NetworkModule {
    type Init = NetworkInit;
    type Input = NetworkMsg;
    type Output = ();
    type CommandOutput = NetworkCmd;

    view! {
        gtk::Box {
            add_css_class: "network",

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
        let network_config = &config.modules.network;

        let (initial_icon, initial_label) = Self::compute_display(network_config, &init.network);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: initial_icon,
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: network_config.icon_color.clone(),
                    label_color: network_config.label_color.clone(),
                    icon_background: network_config.icon_bg_color.clone(),
                    button_background: network_config.button_bg_color.clone(),
                    border_color: network_config.border_color.clone(),
                    auto_icon_color: CssToken::Accent,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: network_config.label_max_length.clone(),
                    show_icon: network_config.icon_show.clone(),
                    show_label: network_config.label_show.clone(),
                    show_border: network_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => NetworkMsg::LeftClick,
                BarButtonOutput::RightClick => NetworkMsg::RightClick,
                BarButtonOutput::MiddleClick => NetworkMsg::MiddleClick,
                BarButtonOutput::ScrollUp => NetworkMsg::ScrollUp,
                BarButtonOutput::ScrollDown => NetworkMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, network_config, &init.network);

        let mut wifi_watcher = WatcherToken::new();
        let mut wired_watcher = WatcherToken::new();

        watchers::spawn_wifi_watchers(&sender, &init.network, wifi_watcher.reset());
        watchers::spawn_wired_watchers(&sender, &init.network, wired_watcher.reset());

        let opener = DropdownOpener::for_button(
            &init.dropdowns,
            &bar_button,
            network_config.clone(),
        );

        let model = Self {
            bar_button,
            config: init.config,
            wifi_watcher,
            wired_watcher,
            network: init.network,
            opener,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &self.config.config().modules.network;

        let action = match msg {
            NetworkMsg::LeftClick => config.left_click.get(),
            NetworkMsg::RightClick => config.right_click.get(),
            NetworkMsg::MiddleClick => config.middle_click.get(),
            NetworkMsg::ScrollUp => config.scroll_up.get(),
            NetworkMsg::ScrollDown => config.scroll_down.get(),
        };

        self.opener.dispatch(&action);
    }

    fn update_cmd(&mut self, msg: NetworkCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        let network_config = &self.config.config().modules.network;

        match msg {
            NetworkCmd::StateChanged | NetworkCmd::IconConfigChanged => {
                let (icon, label) = Self::compute_display(network_config, &self.network);
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            NetworkCmd::WifiDeviceChanged => {
                let token = self.wifi_watcher.reset();
                watchers::spawn_wifi_watchers(&sender, &self.network, token);

                let (icon, label) = Self::compute_display(network_config, &self.network);
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            NetworkCmd::WiredDeviceChanged => {
                let token = self.wired_watcher.reset();
                watchers::spawn_wired_watchers(&sender, &self.network, token);

                let (icon, label) = Self::compute_display(network_config, &self.network);
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
        }
    }
}
