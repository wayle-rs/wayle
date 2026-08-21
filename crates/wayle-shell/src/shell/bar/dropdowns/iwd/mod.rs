mod active_connections;
mod available_networks;
mod factory;
mod helpers;
mod messages;
mod methods;
mod password_form;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_iwd::IwdService;
use wayle_widgets::{WatcherToken, prelude::*};

pub(super) use self::factory::Factory;
use self::{
    active_connections::{ActiveConnections, ActiveConnectionsInit, ActiveConnectionsInput},
    available_networks::{
        AvailableNetworks, AvailableNetworksInit, AvailableNetworksInput, AvailableNetworksOutput,
    },
    messages::{IwdDropdownCmd, IwdDropdownInit, IwdDropdownMsg},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 382.0;
const BASE_HEIGHT: f32 = 512.0;

pub(crate) struct IwdDropdown {
    iwd: Arc<IwdService>,
    scaled_width: i32,
    scaled_height: i32,
    powered: bool,
    scanning: bool,
    station_available: bool,
    active_connections: Controller<ActiveConnections>,
    available_networks: Controller<AvailableNetworks>,
    station_watcher: WatcherToken,
}

#[relm4::component(pub(crate))]
impl Component for IwdDropdown {
    type Init = IwdDropdownInit;
    type Input = IwdDropdownMsg;
    type Output = ();
    type CommandOutput = IwdDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "network-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {
                set_overflow: gtk::Overflow::Hidden,

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("tb-wifi-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-iwd-title"),
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            add_css_class: "network-scan-btn",
                            set_icon_name: "tb-refresh-symbolic",
                            #[watch]
                            set_visible: model.station_available && model.powered,
                            #[watch]
                            set_sensitive: !model.scanning,
                            #[watch]
                            set_css_classes: &if model.scanning {
                                vec!["ghost-icon", "network-scan-btn", "scanning"]
                            } else {
                                vec!["ghost-icon", "network-scan-btn"]
                            },
                            connect_clicked => IwdDropdownMsg::ScanRequested,
                        },

                        #[template]
                        Switch {
                            #[watch]
                            #[block_signal(power_toggle_handler)]
                            set_active: model.powered,
                            #[watch]
                            set_visible: model.station_available,
                            connect_state_set[sender] => move |switch, active| {
                                sender.input(IwdDropdownMsg::PowerToggled(active));
                                switch.set_state(active);
                                gtk::glib::Propagation::Stop
                            } @power_toggle_handler,
                        },
                    },
                },

                #[template]
                DropdownContent {
                    add_css_class: "network-content",
                    set_vexpand: true,

                    #[local_ref]
                    active_connections_widget -> gtk::Box {},

                    #[local_ref]
                    available_networks_widget -> gtk::Box {
                        set_vexpand: true,
                    },
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
                iwd: init.iwd.clone(),
                config: init.config.clone(),
            })
            .detach();

        let available_networks = AvailableNetworks::builder()
            .launch(AvailableNetworksInit {
                iwd: init.iwd.clone(),
                config: init.config.clone(),
            })
            .forward(sender.input_sender(), IwdDropdownMsg::AvailableNetworks);

        let station = init.iwd.station.get();
        let station_available = station.is_some();
        let powered = station.as_ref().is_some_and(|station| station.powered.get());
        let scanning = station.as_ref().is_some_and(|station| station.scanning.get());

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn(&sender, &init.config, &init.iwd);

        let mut model = Self {
            iwd: init.iwd,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            powered,
            scanning,
            station_available,
            active_connections,
            available_networks,
            station_watcher: WatcherToken::new(),
        };

        model.reset_station_watchers(&sender);

        let active_connections_widget = model.active_connections.widget();
        let available_networks_widget = model.available_networks.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            IwdDropdownMsg::PowerToggled(active) => {
                self.toggle_powered(active, &sender);
            }
            IwdDropdownMsg::ScanRequested => {
                self.request_scan(&sender);
            }
            IwdDropdownMsg::AvailableNetworks(output) => match output {
                AvailableNetworksOutput::ConnectionFailed { ssid, message } => {
                    self.active_connections
                        .emit(ActiveConnectionsInput::ShowError { ssid, message });
                }
            },
        }
    }

    fn update_cmd(&mut self, msg: IwdDropdownCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            IwdDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }

            IwdDropdownCmd::StationDeviceChanged => {
                let station = self.iwd.station.get();
                self.station_available = station.is_some();
                self.powered = station.as_ref().is_some_and(|station| station.powered.get());
                self.scanning = station.as_ref().is_some_and(|station| station.scanning.get());

                self.available_networks
                    .emit(AvailableNetworksInput::StationAvailabilityChanged(
                        self.station_available,
                    ));

                self.reset_station_watchers(&sender);
            }

            IwdDropdownCmd::StationFlags { powered, scanning } => {
                self.scanning = scanning;

                if powered != self.powered {
                    self.powered = powered;
                    self.available_networks
                        .emit(AvailableNetworksInput::PoweredChanged(powered));
                }
            }
        }
    }
}
