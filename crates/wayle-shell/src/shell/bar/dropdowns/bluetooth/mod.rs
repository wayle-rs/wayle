mod device_item;
mod factory;
pub(crate) mod helpers;
pub(crate) mod messages;
mod methods;
mod pairing_card;
mod watchers;

use std::{collections::HashMap, sync::Arc, time::Duration};

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_bluetooth::{BluetoothService, core::device::Device};
use wayle_widgets::{WatcherToken, prelude::*};
use zbus::zvariant::OwnedObjectPath;

pub(super) use self::factory::Factory;
use self::{
    device_item::DeviceItem,
    messages::{BluetoothDropdownCmd, BluetoothDropdownInit, BluetoothDropdownMsg},
    pairing_card::{PairingCard, messages::PairingCardInit},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 382.0;
const BASE_HEIGHT: f32 = 512.0;
const SCAN_DURATION: Duration = Duration::from_secs(30);

pub(crate) struct BluetoothDropdown {
    bluetooth: Option<Arc<BluetoothService>>,
    scaled_width: i32,
    scaled_height: i32,
    /// Whether the popover is open; rows are only click-guarded while it is.
    visible: bool,
    my_devices: FactoryVecDeque<DeviceItem>,
    available_devices: FactoryVecDeque<DeviceItem>,
    pairing_card: Controller<PairingCard>,
    state_watcher: WatcherToken,
    /// Property watchers per device, keyed by object path. Dropping an entry
    /// cancels its watcher.
    device_watchers: HashMap<OwnedObjectPath, (Arc<Device>, WatcherToken)>,
}

#[relm4::component(pub(crate))]
impl Component for BluetoothDropdown {
    type Init = BluetoothDropdownInit;
    type Input = BluetoothDropdownMsg;
    type Output = ();
    type CommandOutput = BluetoothDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &[
                "dropdown",
                "bluetooth-dropdown",
            ],
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
                        #[watch]
                        set_icon_name: Some(
                            if model.enabled() {
                                "ld-bluetooth-symbolic"
                            } else {
                                "ld-bluetooth-off-symbolic"
                            }
                        ),
                    },
                    #[template_child]
                    label {
                        set_label: &t!(
                            "dropdown-bluetooth-title"
                        ),
                    },
                    #[template_child]
                    actions {
                        #[template]
                        GhostIconButton {
                            add_css_class:
                                "bluetooth-scan-btn",
                            set_icon_name:
                                "tb-refresh-symbolic",
                            #[watch]
                            set_visible:
                                model.available()
                                    && model.powered(),
                            #[watch]
                            set_sensitive:
                                !model.scanning(),
                            #[watch]
                            set_css_classes: &if
                                model.scanning()
                            {
                                vec![
                                    "ghost-icon",
                                    "bluetooth-scan-btn",
                                    "scanning",
                                ]
                            } else {
                                vec![
                                    "ghost-icon",
                                    "bluetooth-scan-btn",
                                ]
                            },
                            connect_clicked =>
                                BluetoothDropdownMsg::ScanRequested,
                        },

                        // The knob shows what was last asked for; the
                        // switch's state (its colour) is BlueZ's `enabled`.
                        // The knob is put back to BlueZ's state once BlueZ
                        // answers (see `update_cmd_with_view`).
                        #[name = "bt_switch"]
                        #[template]
                        Switch {
                            #[watch]
                            set_state: model.enabled(),
                            #[watch]
                            set_visible: model.available(),
                            #[watch]
                            set_sensitive: !model.hardware_blocked(),
                            // Straight to the service: going through
                            // `update` would re-render the switch from
                            // `enabled` before BlueZ has answered.
                            connect_state_set[bluetooth] =>
                                move |_switch, active|
                            {
                                if let Some(bluetooth) = bluetooth.get() {
                                    if active {
                                        bluetooth.enable();
                                    } else {
                                        bluetooth.disable();
                                    }
                                }
                                gtk::glib::Propagation::Stop
                            } @bt_toggle,
                        },
                    },
                },

                #[template]
                DropdownContent {
                    add_css_class: "bluetooth-content",

                    gtk::ScrolledWindow {
                        add_css_class: "bluetooth-scroll",
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            #[local_ref]
                            pairing_card_widget -> gtk::Box {},

                            #[name = "my_devices_label"]
                            gtk::Label {
                                add_css_class: "section-label",
                                set_halign: gtk::Align::Start,
                                set_label: &t!(
                                    "dropdown-bluetooth-my-devices"
                                ),
                                #[watch]
                                set_visible: model.powered()
                                    && !model
                                        .my_devices
                                        .is_empty(),
                            },

                            #[name = "my_devices_card"]
                            #[template]
                            Card {
                                add_css_class:
                                    "bluetooth-device-list",
                                #[watch]
                                set_visible: model.powered()
                                    && !model
                                        .my_devices
                                        .is_empty(),
                                #[local_ref]
                                my_devices_widget -> gtk::Box {
                                    set_orientation:
                                        gtk::Orientation::Vertical,
                                },
                            },

                            #[name = "available_devices_label"]
                            gtk::Label {
                                add_css_class: "section-label",
                                set_halign: gtk::Align::Start,
                                set_label: &t!(
                                    "dropdown-bluetooth-available-devices"
                                ),
                                #[watch]
                                set_visible: model.powered()
                                    && (!model
                                        .available_devices
                                        .is_empty()
                                        || model.scanning()),
                            },

                            #[name = "available_devices_card"]
                            #[template]
                            Card {
                                add_css_class:
                                    "bluetooth-device-list",
                                #[watch]
                                set_visible: model.powered()
                                    && !model
                                        .available_devices
                                        .is_empty(),
                                #[local_ref]
                                available_devices_widget -> gtk::Box {
                                    set_orientation:
                                        gtk::Orientation::Vertical,
                                },
                            },

                            #[name = "scanning_hint"]
                            gtk::Label {
                                add_css_class:
                                    "bluetooth-no-new-devices",
                                #[watch]
                                set_visible: model.powered()
                                    && model.scanning()
                                    && model
                                        .my_devices
                                        .is_empty()
                                    && model
                                        .available_devices
                                        .is_empty(),
                                set_label: &t!(
                                    "dropdown-bluetooth-no-new"
                                ),
                            },

                            #[name = "empty_no_devices"]
                            #[template]
                            EmptyState {
                                #[watch]
                                set_visible: model.powered()
                                    && !model.scanning()
                                    && model
                                        .my_devices
                                        .is_empty()
                                    && model
                                        .available_devices
                                        .is_empty(),
                                #[template_child]
                                icon {
                                    set_icon_name: Some(
                                        "ld-bluetooth-searching-symbolic"
                                    ),
                                },
                                #[template_child]
                                title {
                                    set_label: &t!(
                                        "dropdown-bluetooth-no-devices-title"
                                    ),
                                },
                                #[template_child]
                                description {
                                    set_label: &t!(
                                        "dropdown-bluetooth-no-devices-description"
                                    ),
                                },
                            },

                            // While not powered; the lists show otherwise.
                            #[name = "empty_bt_off"]
                            #[template]
                            EmptyState {
                                #[watch]
                                set_visible: !model.powered()
                                    && model.available(),
                                #[template_child]
                                icon {
                                    set_icon_name: Some(
                                        "ld-bluetooth-off-symbolic"
                                    ),
                                },
                                #[template_child]
                                title {
                                    #[watch]
                                    set_label: &if model.hardware_blocked() {
                                        t!("dropdown-bluetooth-blocked-title")
                                    } else if model.enabled() {
                                        t!("dropdown-bluetooth-turning-on-title")
                                    } else {
                                        t!("dropdown-bluetooth-off-title")
                                    },
                                },
                                #[template_child]
                                description {
                                    #[watch]
                                    set_visible: !model.enabled(),
                                    #[watch]
                                    set_label: &if model.hardware_blocked() {
                                        t!("dropdown-bluetooth-blocked-description")
                                    } else {
                                        t!("dropdown-bluetooth-off-description")
                                    },
                                },
                            },

                            #[name = "empty_no_adapter"]
                            #[template]
                            EmptyState {
                                #[watch]
                                set_visible: !model.available(),
                                #[template_child]
                                icon {
                                    set_icon_name: Some(
                                        "ld-bluetooth-off-symbolic"
                                    ),
                                },
                                #[template_child]
                                title {
                                    set_label: &t!(
                                        "dropdown-bluetooth-no-adapter-title"
                                    ),
                                },
                                #[template_child]
                                description {
                                    set_label: &t!(
                                        "dropdown-bluetooth-no-adapter-description"
                                    ),
                                },
                            },
                        },
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
        let my_devices = Self::build_device_list();
        let available_devices = Self::build_device_list();

        let pairing_card = PairingCard::builder()
            .launch(PairingCardInit {
                bluetooth: init.bluetooth.clone(),
            })
            .detach();

        let scale = init.config.config().styling.scale.get().value();

        watchers::spawn_config_watcher(&sender, &init.config);
        watchers::spawn_service_watcher(&sender, &init.bluetooth);

        let model = Self {
            bluetooth: None,
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            visible: root.is_visible(),
            my_devices,
            available_devices,
            pairing_card,
            state_watcher: WatcherToken::new(),
            device_watchers: HashMap::new(),
        };

        let input_sender = sender.input_sender().clone();
        root.connect_visible_notify(move |popover| {
            input_sender.emit(BluetoothDropdownMsg::VisibilityChanged(
                popover.is_visible(),
            ));
        });

        let bluetooth = init.bluetooth.clone();
        let pairing_card_widget = model.pairing_card.widget();
        let my_devices_widget = model.my_devices.widget();
        let available_devices_widget = model.available_devices.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BluetoothDropdownMsg::VisibilityChanged(visible) => {
                self.handle_visibility_changed(visible);
            }

            BluetoothDropdownMsg::ScanRequested => {
                self.handle_scan_requested();
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: BluetoothDropdownCmd,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        // The knob shows what was last asked for until BlueZ answers: with a
        // change of state, or with a failure recorded on an adapter, which may
        // change nothing. Either way the knob goes back to BlueZ's state.
        let answered = matches!(
            msg,
            BluetoothDropdownCmd::ServiceReady(_) | BluetoothDropdownCmd::StateChanged
        );

        self.update_cmd(msg, sender.clone(), root);
        self.update_view(widgets, sender);

        // Only a knob that differs: moving one mid-animation would cancel
        // the click that started the animation.
        if answered && widgets.bt_switch.is_active() != self.enabled() {
            widgets.bt_switch.block_signal(&widgets.bt_toggle);
            widgets.bt_switch.set_active(self.enabled());
            widgets.bt_switch.unblock_signal(&widgets.bt_toggle);
        }
    }

    fn update_cmd(
        &mut self,
        msg: BluetoothDropdownCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            BluetoothDropdownCmd::ServiceReady(bt) => {
                let token = self.state_watcher.reset();
                watchers::spawn_bt_watchers(&sender, &bt, token);

                self.bluetooth = Some(bt);
                self.device_watchers.clear();
                self.sync_device_watchers(&sender);
                self.rebuild_device_lists();
            }

            BluetoothDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }

            // The view reads the service's state afresh.
            BluetoothDropdownCmd::StateChanged | BluetoothDropdownCmd::RadioBlockChanged => {}

            BluetoothDropdownCmd::DevicesChanged => {
                if self.sync_device_watchers(&sender) {
                    self.rebuild_device_lists();
                }
            }

            BluetoothDropdownCmd::DevicePropertyChanged => {
                self.rebuild_device_lists();
            }

            BluetoothDropdownCmd::PairingRequested {
                request,
                new_prompt,
            } => {
                self.handle_pairing_request(request, new_prompt);
            }
        }
    }
}
