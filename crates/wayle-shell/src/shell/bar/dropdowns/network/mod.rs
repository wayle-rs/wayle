mod active_connections;
mod available_networks;
mod factory;
pub(crate) mod helpers;
mod messages;
mod methods;
mod password_form;
mod vpn_tunnels;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_network::NetworkService;
use wayle_widgets::{WatcherToken, prelude::*};

pub(super) use self::factory::Factory;
use self::{
    active_connections::{ActiveConnections, ActiveConnectionsInit, ActiveConnectionsInput},
    available_networks::{
        AvailableNetworks, AvailableNetworksInit, AvailableNetworksInput, AvailableNetworksOutput,
    },
    messages::{NetworkDropdownCmd, NetworkDropdownInit, NetworkDropdownMsg},
    vpn_tunnels::{VpnTunnels, VpnTunnelsInit},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 382.0;

pub(crate) struct NetworkDropdown {
    network: Arc<NetworkService>,
    scaled_width: i32,
    wifi_enabled: bool,
    wifi_available: bool,
    scanning: bool,
    active_connections: Controller<ActiveConnections>,
    vpn_tunnels: Controller<VpnTunnels>,
    available_networks: Controller<AvailableNetworks>,
    wifi_watcher: WatcherToken,
}

#[relm4::component(pub(crate))]
impl Component for NetworkDropdown {
    type Init = NetworkDropdownInit;
    type Input = NetworkDropdownMsg;
    type Output = ();
    type CommandOutput = NetworkDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "network-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        #[watch]
                        set_icon_name: Some(if model.wifi_available {
                            "tb-wifi-symbolic"
                        } else {
                            "cm-wired-symbolic"
                        }),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-network-title"),
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            add_css_class: "network-scan-btn",
                            set_icon_name: "tb-refresh-symbolic",
                            #[watch]
                            set_visible: model.wifi_available && model.wifi_enabled,
                            #[watch]
                            set_sensitive: !model.scanning,
                            #[watch]
                            set_css_classes: &if model.scanning {
                                vec!["ghost-icon", "network-scan-btn", "scanning"]
                            } else {
                                vec!["ghost-icon", "network-scan-btn"]
                            },
                            connect_clicked => NetworkDropdownMsg::ScanRequested,
                        },

                        #[template]
                        Switch {
                            #[watch]
                            #[block_signal(wifi_toggle_handler)]
                            set_active: model.wifi_enabled,
                            #[watch]
                            set_visible: model.wifi_available,
                            connect_state_set[sender] => move |switch, active| {
                                sender.input(NetworkDropdownMsg::WifiToggled(active));
                                switch.set_state(active);
                                gtk::glib::Propagation::Stop
                            } @wifi_toggle_handler,
                        },
                    },
                },

                #[template]
                DropdownContent {
                    add_css_class: "network-content",

                    #[local_ref]
                    active_connections_widget -> gtk::Box {},

                    #[local_ref]
                    vpn_tunnels_widget -> gtk::Box {},

                    #[local_ref]
                    available_networks_widget -> gtk::Box {},
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let active_connections = ActiveConnections::builder()
            .launch(ActiveConnectionsInit {
                network: init.network.clone(),
            })
            .detach();

        let vpn_tunnels = VpnTunnels::builder()
            .launch(VpnTunnelsInit {
                network: init.network.clone(),
                config: init.config.clone(),
                provider: vpn_tunnels::providers::wireguard_config(),
            })
            .forward(sender.input_sender(), NetworkDropdownMsg::VpnTunnels);

        let available_networks = AvailableNetworks::builder()
            .launch(AvailableNetworksInit {
                network: init.network.clone(),
            })
            .forward(sender.input_sender(), NetworkDropdownMsg::AvailableNetworks);

        let wifi = init.network.wifi.get();
        let wifi_available = wifi.is_some();
        let wifi_enabled = wifi.as_ref().is_some_and(|wifi| wifi.enabled.get());

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn(&sender, &init.config, &init.network);

        let mut model = Self {
            network: init.network,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            wifi_enabled,
            wifi_available,
            scanning: false,
            active_connections,
            vpn_tunnels,
            available_networks,
            wifi_watcher: WatcherToken::new(),
        };

        model.reset_wifi_watchers(&sender);

        let active_connections_widget = model.active_connections.widget();
        let vpn_tunnels_widget = model.vpn_tunnels.widget();
        let available_networks_widget = model.available_networks.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            NetworkDropdownMsg::WifiToggled(active) => {
                self.toggle_wifi(active, &sender);
            }
            NetworkDropdownMsg::ScanRequested => {
                self.available_networks
                    .emit(AvailableNetworksInput::ScanRequested);
            }
            NetworkDropdownMsg::VpnTunnels(_output) => {
                // VPN tunnel events are handled internally by the component
            }
            NetworkDropdownMsg::AvailableNetworks(output) => match output {
                AvailableNetworksOutput::ScanStarted => {
                    self.scanning = true;
                }

                AvailableNetworksOutput::ScanComplete => {
                    self.scanning = false;
                }

                AvailableNetworksOutput::Connecting(ssid) => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::SetConnecting(ssid));
                }

                AvailableNetworksOutput::ConnectionProgress(step) => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::SetConnectingStep(step));
                }

                AvailableNetworksOutput::ClearConnecting => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::ClearConnecting);
                }

                AvailableNetworksOutput::Connected => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::ClearConnecting);

                    self.active_connections
                        .emit(ActiveConnectionsInput::ClearConnectionError);
                }

                AvailableNetworksOutput::ConnectionFailed(err) => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::SetConnectionError(err));
                }
            },
        }
    }

    fn update_cmd(
        &mut self,
        msg: NetworkDropdownCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            NetworkDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }

            NetworkDropdownCmd::WifiDeviceChanged => {
                let wifi = self.network.wifi.get();
                self.wifi_available = wifi.is_some();
                self.wifi_enabled = wifi.as_ref().is_some_and(|wifi| wifi.enabled.get());

                self.available_networks
                    .emit(AvailableNetworksInput::WifiAvailabilityChanged(
                        self.wifi_available,
                    ));

                self.reset_wifi_watchers(&sender);
            }

            NetworkDropdownCmd::WifiEnabledChanged(enabled) => {
                self.wifi_enabled = enabled;

                self.available_networks
                    .emit(AvailableNetworksInput::WifiEnabledChanged(enabled));
            }
        }
    }
}
