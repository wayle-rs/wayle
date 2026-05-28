mod device_item;
mod factory;
mod helpers;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use wayle_brightness::BacklightDevice;
use wayle_widgets::{WatcherToken, prelude::*};

pub(super) use self::factory::Factory;
use self::{
    device_item::{BrightnessDeviceItem, messages::BrightnessDeviceItemOutput},
    messages::{BrightnessDropdownCmd, BrightnessDropdownInit, BrightnessDropdownInput},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 320.0;

pub(crate) struct BrightnessDropdown {
    devices: Vec<Arc<BacklightDevice>>,
    device_items: FactoryVecDeque<BrightnessDeviceItem>,
    devices_watcher: WatcherToken,
    scaled_width: i32,
}

#[relm4::component(pub(crate))]
impl Component for BrightnessDropdown {
    type Init = BrightnessDropdownInit;
    type Input = BrightnessDropdownInput;
    type Output = ();
    type CommandOutput = BrightnessDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "brightness-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("ld-sun-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-brightness-title"),
                    },
                    #[template_child]
                    actions {
                        set_visible: false,
                    },
                },

                #[template]
                DropdownContent {
                    set_vexpand: true,

                    gtk::Box {
                        add_css_class: "brightness-devices",
                        set_orientation: gtk::Orientation::Vertical,
                        #[watch]
                        set_visible: !model.devices.is_empty(),

                        #[local_ref]
                        device_item_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                        },
                    },

                    #[template]
                    EmptyState {
                        add_css_class: "brightness-empty",
                        #[watch]
                        set_visible: model.devices.is_empty(),
                        #[template_child]
                        icon {
                            add_css_class: "sm",
                            set_icon_name: Some("ld-sun-symbolic"),
                        },
                        #[template_child]
                        title {
                            set_label: &t!("dropdown-brightness-empty-title"),
                        },
                        #[template_child]
                        description {
                            set_label: &t!("dropdown-brightness-empty-description"),
                            set_wrap: true,
                            set_wrap_mode: gtk::pango::WrapMode::WordChar,
                            set_justify: gtk::Justification::Center,
                            set_max_width_chars: 32,
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
        let devices = init.brightness.devices.get();

        let device_items = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |item_output| match item_output {
                BrightnessDeviceItemOutput::BrightnessChanged(device_name, percentage) => {
                    BrightnessDropdownInput::DeviceBrightnessChanged(device_name, percentage)
                }
            });

        watchers::spawn_top_level(&sender, &init.brightness, &init.config);

        let scale = init.config.config().styling.scale.get().value();

        let mut model = Self {
            devices,
            device_items,
            devices_watcher: WatcherToken::new(),
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
        };

        model.sync_devices();
        model.resume_device_watchers(&sender);

        let device_item_list = model.device_items.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BrightnessDropdownInput::DeviceBrightnessChanged(device_name, percentage) => {
                self.commit_brightness(&device_name, percentage, &sender);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: BrightnessDropdownCmd,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            BrightnessDropdownCmd::DevicesChanged(devices) => {
                self.devices = devices;
                self.sync_devices();
                self.resume_device_watchers(&sender);
            }
            BrightnessDropdownCmd::DeviceBrightnessUpdated(device_name) => {
                self.sync_single_device(&device_name);
            }
            BrightnessDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
            }
        }
    }
}

impl BrightnessDropdown {
    fn resume_device_watchers(&mut self, sender: &ComponentSender<Self>) {
        let token = self.devices_watcher.reset();
        watchers::spawn_per_device(sender, &self.devices, token);
    }
}
