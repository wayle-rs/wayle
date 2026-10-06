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
    active_connections::{ActiveConnections, ActiveConnectionsInit},
    available_networks::{AvailableNetworks, AvailableNetworksInit, AvailableNetworksInput},
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
                        set_icon_name: Some("network-wireless-symbolic"),
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

                        // GtkSwitch's delayed state: the knob (`active`)
                        // shows what was last asked for, and moves at once;
                        // the colour (`state`) is the station's `powered`
                        // (adapter unblocked and device up), as IWD reports
                        // it. The knob is put back to IWD's state once IWD
                        // answers (see `update_cmd_with_view`).
                        #[name = "power_switch"]
                        #[template]
                        Switch {
                            #[watch]
                            set_state: model.powered,
                            #[watch]
                            set_visible: model.station_available,
                            // Stops GTK from setting `state` itself.
                            connect_state_set[iwd] => move |_switch, active| {
                                if let Some(station) = iwd.station.get() {
                                    station.set_powered(active);
                                }
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
        root: Self::Root,
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
            .detach();

        let station = init.iwd.station.get();
        let station_available = station.is_some();
        let powered = station
            .as_ref()
            .is_some_and(|station| station.powered.get());
        let scanning = station
            .as_ref()
            .is_some_and(|station| station.scanning.get());

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

        let list_sender = model.available_networks.sender().clone();
        list_sender.emit(AvailableNetworksInput::VisibilityChanged(root.is_visible()));
        let shown_sender = sender.input_sender().clone();
        root.connect_visible_notify(move |popover| {
            list_sender.emit(AvailableNetworksInput::VisibilityChanged(
                popover.is_visible(),
            ));
            if popover.is_visible() {
                shown_sender.emit(IwdDropdownMsg::Shown);
            }
        });

        let iwd = model.iwd.clone();
        let active_connections_widget = model.active_connections.widget();
        let available_networks_widget = model.available_networks.widget();
        let widgets = view_output!();
        model.sync_knob(&widgets);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            IwdDropdownMsg::ScanRequested => self.request_scan(),
            // In case a request ended without an answer to resync on (e.g.
            // IWD failing to bring the device up after an unblock).
            IwdDropdownMsg::Shown => self.sync_knob(widgets),
        }
        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: IwdDropdownCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        // IWD answered a power request, with a change of state or a recorded
        // failure (or another client changed it). Not on other updates, such
        // as a scan ending, which can arrive before the answer.
        let answered = matches!(
            msg,
            IwdDropdownCmd::StationDeviceChanged | IwdDropdownCmd::PowerAnswered(_)
        );

        self.update_cmd(msg, sender.clone(), root);
        self.update_view(widgets, sender);
        if answered {
            self.sync_knob(widgets);
        }
    }

    fn update_cmd(
        &mut self,
        msg: IwdDropdownCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            IwdDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }

            IwdDropdownCmd::StationDeviceChanged => {
                let station = self.iwd.station.get();
                self.station_available = station.is_some();
                self.powered = station
                    .as_ref()
                    .is_some_and(|station| station.powered.get());
                self.scanning = station
                    .as_ref()
                    .is_some_and(|station| station.scanning.get());

                self.reset_station_watchers(&sender);
            }

            IwdDropdownCmd::ScanningChanged(scanning) => {
                self.scanning = scanning;
            }

            IwdDropdownCmd::PowerAnswered(powered) => {
                self.powered = powered;
            }
        }
    }
}
