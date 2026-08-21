mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_bluetooth::BluetoothService;
use wayle_core::DeferredService;
use wayle_iwd::IwdService;
use wayle_network::NetworkService;
use wayle_notification::NotificationService;
use wayle_power_profiles::{PowerProfilesService, types::profile::PowerProfile};
use wayle_widgets::WatcherToken;

pub(crate) use self::messages::QuickActionsInit;
use self::messages::{QuickActionsCmd, QuickActionsInput};
use crate::{i18n::t, services::IdleInhibitService};

pub(crate) struct QuickActionsSection {
    network: Option<Arc<NetworkService>>,
    iwd: Option<Arc<IwdService>>,
    bluetooth: DeferredService<BluetoothService>,
    notification: Option<Arc<NotificationService>>,
    power_profiles: DeferredService<PowerProfilesService>,
    idle_inhibit: Arc<IdleInhibitService>,

    power_profile_token: WatcherToken,
    wifi_enabled_token: WatcherToken,

    wifi_active: bool,
    bluetooth_active: bool,
    airplane_active: bool,
    dnd_active: bool,
    idle_inhibit_active: bool,
    power_saver_active: bool,

    has_wifi: bool,
    has_bluetooth: bool,
    has_notification: bool,
    has_power_profiles: bool,

    pre_airplane_wifi: bool,
    pre_airplane_bt: bool,
}

#[relm4::component(pub(crate))]
impl Component for QuickActionsSection {
    type Init = QuickActionsInit;
    type Input = QuickActionsInput;
    type Output = ();
    type CommandOutput = QuickActionsCmd;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["card", "dashboard-card"],

            #[name = "actions_grid"]
            gtk::Grid {
                add_css_class: "quick-actions",
                set_hexpand: true,
                set_column_homogeneous: true,
                set_row_homogeneous: true,
                set_row_spacing: 4,
                set_column_spacing: 8,

                #[name = "wifi_btn"]
                attach[0, 0, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.wifi_active),
                    #[watch]
                    set_sensitive: model.has_wifi && !model.airplane_active,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::WifiToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                #[watch]
                                set_icon_name: Some(if model.wifi_active {
                                    "ld-wifi-symbolic"
                                } else {
                                    "ld-wifi-off-symbolic"
                                }),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-wifi"),
                        },
                    },
                },

                #[name = "bt_btn"]
                attach[1, 0, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.bluetooth_active),
                    #[watch]
                    set_sensitive: model.has_bluetooth && !model.airplane_active,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::BluetoothToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                #[watch]
                                set_icon_name: Some(if model.bluetooth_active {
                                    "ld-bluetooth-symbolic"
                                } else {
                                    "ld-bluetooth-off-symbolic"
                                }),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-bluetooth"),
                        },
                    },
                },

                #[name = "airplane_btn"]
                attach[2, 0, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.airplane_active),
                    #[watch]
                    set_sensitive: model.has_wifi || model.has_bluetooth,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::AirplaneToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                set_icon_name: Some("ld-plane-symbolic"),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-airplane"),
                        },
                    },
                },

                #[name = "dnd_btn"]
                attach[0, 1, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.dnd_active),
                    #[watch]
                    set_sensitive: model.has_notification,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::DndToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                #[watch]
                                set_icon_name: Some(if model.dnd_active {
                                    "ld-bell-off-symbolic"
                                } else {
                                    "ld-bell-symbolic"
                                }),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-dnd"),
                        },
                    },
                },

                #[name = "idle_btn"]
                attach[1, 1, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.idle_inhibit_active),
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::IdleInhibitToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                set_icon_name: Some("ld-eye-symbolic"),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-idle-inhibit"),
                        },
                    },
                },

                #[name = "power_saver_btn"]
                attach[2, 1, 1, 1] = &gtk::Button {
                    add_css_class: "quick-action",
                    #[watch]
                    set_class_active: ("active", model.power_saver_active),
                    #[watch]
                    set_sensitive: model.has_power_profiles,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => QuickActionsInput::PowerSaverToggled,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,

                        gtk::Box {
                            add_css_class: "quick-action-icon",
                            set_halign: gtk::Align::Center,

                            gtk::Image {
                                set_icon_name: Some("ld-leaf-symbolic"),
                            },
                        },

                        gtk::Label {
                            add_css_class: "quick-action-label",
                            set_halign: gtk::Align::Center,
                            set_label: &t!("dropdown-dashboard-power-saver"),
                        },
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
        let has_wifi = methods::wifi_enabled_property(&init.network, &init.iwd).is_some();

        let current_bt = init.bluetooth.get();

        let has_bluetooth = current_bt
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.available.get());

        let bluetooth_active = current_bt
            .as_ref()
            .is_some_and(|bluetooth| bluetooth.enabled.get());

        let has_notification = init.notification.is_some();

        let current_pp = init.power_profiles.get();
        let has_power_profiles = current_pp.is_some();
        let power_saver_active = current_pp.as_ref().is_some_and(|service| {
            service.power_profiles.active_profile.get() == PowerProfile::PowerSaver
        });

        watchers::spawn(
            &sender,
            &init.network,
            &init.iwd,
            &init.bluetooth,
            &init.notification,
            &init.power_profiles,
            &init.idle_inhibit,
        );

        let mut power_profile_token = WatcherToken::new();
        if let Some(service) = &current_pp {
            let token = power_profile_token.reset();
            watchers::spawn_power_profile_watcher(&sender, service, token);
        }

        let mut wifi_enabled_token = WatcherToken::new();
        let wifi_active = methods::wifi_enabled_property(&init.network, &init.iwd)
            .is_some_and(|enabled| {
                let token = wifi_enabled_token.reset();
                watchers::spawn_wifi_enabled_watcher(&sender, &enabled, token);
                enabled.get()
            });

        let model = Self {
            network: init.network,
            iwd: init.iwd,
            bluetooth: init.bluetooth,
            notification: init.notification,
            power_profiles: init.power_profiles,
            idle_inhibit: init.idle_inhibit,

            power_profile_token,
            wifi_enabled_token,

            wifi_active,
            bluetooth_active,
            airplane_active: false,
            dnd_active: false,
            idle_inhibit_active: false,
            power_saver_active,

            has_wifi,
            has_bluetooth,
            has_notification,
            has_power_profiles,

            pre_airplane_wifi: false,
            pre_airplane_bt: false,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            QuickActionsInput::WifiToggled => self.toggle_wifi(&sender),
            QuickActionsInput::BluetoothToggled => self.toggle_bluetooth(&sender),
            QuickActionsInput::AirplaneToggled => self.toggle_airplane(&sender),
            QuickActionsInput::DndToggled => self.toggle_dnd(&sender),
            QuickActionsInput::IdleInhibitToggled => self.toggle_idle_inhibit(),
            QuickActionsInput::PowerSaverToggled => self.toggle_power_saver(&sender),
        }
    }

    fn update_cmd(
        &mut self,
        msg: QuickActionsCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            QuickActionsCmd::WifiChanged(active) => self.wifi_active = active,
            QuickActionsCmd::WifiAvailabilityChanged(available) => {
                self.has_wifi = available;
                if available {
                    if let Some(enabled) = methods::wifi_enabled_property(&self.network, &self.iwd) {
                        self.wifi_active = enabled.get();
                        let token = self.wifi_enabled_token.reset();
                        watchers::spawn_wifi_enabled_watcher(&sender, &enabled, token);
                    }
                } else {
                    self.wifi_active = false;
                    self.wifi_enabled_token = WatcherToken::new();
                }
            }

            QuickActionsCmd::BluetoothChanged(active) => self.bluetooth_active = active,

            QuickActionsCmd::BluetoothAvailabilityChanged(available) => {
                self.has_bluetooth = available;
                if !available {
                    self.bluetooth_active = false;
                }
            }

            QuickActionsCmd::DndChanged(active) => self.dnd_active = active,
            QuickActionsCmd::IdleInhibitChanged(active) => self.idle_inhibit_active = active,
            QuickActionsCmd::PowerSaverChanged(active) => self.power_saver_active = active,

            QuickActionsCmd::BluetoothReady(service) => {
                self.has_bluetooth = service.available.get();
                self.bluetooth_active = service.enabled.get();

                watchers::spawn_bluetooth_watchers(&sender, &service);
            }

            QuickActionsCmd::PowerProfilesReady(service) => {
                self.has_power_profiles = true;
                self.power_saver_active =
                    service.power_profiles.active_profile.get() == PowerProfile::PowerSaver;

                let token = self.power_profile_token.reset();
                watchers::spawn_power_profile_watcher(&sender, &service, token);
            }
        }
    }
}
