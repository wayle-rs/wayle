mod helpers;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_battery::{
    BatteryService,
    types::{DeviceState, WarningLevel},
};
use wayle_widgets::prelude::*;

pub(crate) use self::messages::BatterySectionInit;
use self::messages::{BatterySectionCmd, BatterySectionInput};
use crate::i18n::t;

pub(crate) struct BatterySection {
    battery: Arc<BatteryService>,

    percentage: f64,
    state: DeviceState,
    time_to_empty: i64,
    time_to_full: i64,
    energy_rate: f64,
    energy: f64,
    energy_full: f64,
    capacity: f64,
    warning_level: WarningLevel,
    is_present: bool,

    charge_end_threshold: u32,
    charge_threshold_supported: bool,
    charge_threshold_enabled: bool,
}

#[relm4::component(pub(crate))]
impl Component for BatterySection {
    type Init = BatterySectionInit;
    type Input = BatterySectionInput;
    type Output = ();
    type CommandOutput = BatterySectionCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            if !model.is_present {
                #[template]
                EmptyState {
                    #[template_child]
                    icon {
                        set_icon_name: Some("ld-unplug-symbolic"),
                    },
                    #[template_child]
                    title {
                        set_label: &t!("dropdown-battery-no-battery-title"),
                    },
                    #[template_child]
                    description {
                        set_label: &t!("dropdown-battery-no-battery-description"),
                    },
                }
            } else {
                #[name = "battery_content"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "hero"]
                    gtk::Box {
                        add_css_class: "battery-hero",

                        #[name = "hero_percentage"]
                        gtk::Label {
                            add_css_class: "battery-hero-pct",
                            #[watch]
                            set_css_classes: &[
                                "battery-hero-pct",
                                helpers::hero_pct_class(
                                    model.percentage,
                                    &model.warning_level,
                                ),
                            ],
                            #[watch]
                            set_label: &format!("{}%", model.percentage as u32),
                        },

                        #[name = "hero_meta"]
                        gtk::Box {
                            add_css_class: "battery-hero-meta",
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,

                            #[name = "state_label"]
                            gtk::Label {
                                add_css_class: "battery-hero-state",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_css_classes: &[
                                    "battery-hero-state",
                                    helpers::hero_state_class(
                                        &model.warning_level,
                                        model.is_charging(),
                                    ),
                                ],
                                #[watch]
                                set_label: &model.state_label(),
                            },

                            gtk::Label {
                                add_css_class: "battery-hero-time",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &model.time_display(),
                                #[watch]
                                set_visible: model.has_time_display(),
                            },

                            gtk::Label {
                                add_css_class: "battery-hero-time",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &model.input_display(),
                                #[watch]
                                set_visible: model.is_charging() && model.energy_rate > 0.0,
                            },
                        },
                    },

                    #[name = "gauge"]
                    gtk::LevelBar {
                        add_css_class: "battery-gauge",
                        set_mode: gtk::LevelBarMode::Continuous,
                        set_min_value: 0.0,
                        set_max_value: 100.0,
                        #[watch]
                        set_value: model.percentage.clamp(0.0, 100.0),
                        #[watch]
                        set_css_classes: &[
                            "battery-gauge",
                            helpers::gauge_class(model.percentage, &model.warning_level),
                        ],
                    },

                    #[name = "details"]
                    gtk::CenterBox {
                        add_css_class: "battery-details",

                        #[wrap(Some)]
                        #[name = "draw_detail"]
                        set_start_widget = &gtk::Box {
                            add_css_class: "battery-detail",
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Label {
                                add_css_class: "battery-detail-value",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &model.draw_value(),
                            },
                            gtk::Label {
                                add_css_class: "battery-detail-label",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &model.draw_label(),
                            },
                        },

                        #[wrap(Some)]
                        #[name = "capacity_detail"]
                        set_center_widget = &gtk::Box {
                            add_css_class: "battery-detail",
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Label {
                                add_css_class: "battery-detail-value",
                                #[watch]
                                set_label: &model.capacity_value(),
                            },
                            gtk::Label {
                                add_css_class: "battery-detail-label",
                                #[watch]
                                set_label: &model.capacity_label(),
                            },
                        },

                        #[wrap(Some)]
                        #[name = "health_detail"]
                        set_end_widget = &gtk::Box {
                            add_css_class: "battery-detail",
                            set_orientation: gtk::Orientation::Vertical,

                            #[name = "health_indicator"]
                            gtk::Box {
                                set_halign: gtk::Align::End,

                                gtk::Box {
                                    add_css_class: "health-dot",
                                    set_valign: gtk::Align::Center,
                                    set_halign: gtk::Align::Center,
                                    #[watch]
                                    set_css_classes: &[
                                        "health-dot",
                                        helpers::health_class(model.capacity),
                                    ],
                                },

                                gtk::Label {
                                    add_css_class: "battery-detail-value",
                                    #[watch]
                                    set_label: &helpers::health_value(model.capacity),
                                },
                            },
                            gtk::Label {
                                add_css_class: "battery-detail-label",
                                set_halign: gtk::Align::End,
                                set_label: &t!("dropdown-battery-health"),
                            },
                        },
                    },

                    #[name = "charge_limit_label"]
                    gtk::Label {
                        add_css_class: "section-label",
                        set_label: &t!("dropdown-battery-charge-limit"),
                        set_halign: gtk::Align::Start,
                    },

                    #[name = "charge_limit_card"]
                    gtk::Box {
                        add_css_class: "charge-limit",
                        #[watch]
                        set_visible: model.charge_threshold_supported,

                        #[name = "charge_limit_info"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,

                            gtk::Label {
                                add_css_class: "charge-limit-title",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &if model.charge_end_threshold == 0 {
                                    t!("dropdown-battery-charge-limit-optimized")
                                } else {
                                    t!(
                                        "dropdown-battery-limit-to",
                                        threshold = model.charge_end_threshold.to_string()
                                    )
                                },
                            },

                            gtk::Label {
                                add_css_class: "charge-limit-subtitle",
                                set_halign: gtk::Align::Start,
                                #[watch]
                                set_label: &if model.charge_end_threshold == 0 {
                                    t!("dropdown-battery-charge-limit-optimized-subtitle")
                                } else {
                                    t!(
                                        "dropdown-battery-resumes-at",
                                        threshold = model.resume_threshold().to_string()
                                    )
                                },
                            },
                        },

                        #[template]
                        Switch {
                            set_valign: gtk::Align::Center,
                            #[watch]
                            #[block_signal(limit_handler)]
                            set_active: model.charge_threshold_enabled,
                            connect_state_set[sender] => move |switch, active| {
                                sender.input(BatterySectionInput::ChargeLimitToggled(active));
                                switch.set_state(active);
                                gtk::glib::Propagation::Stop
                            } @limit_handler,
                        },
                    },

                    #[name = "charge_limit_unsupported"]
                    gtk::Box {
                        add_css_class: "charge-limit-not-supported",
                        #[watch]
                        set_visible: !model.charge_threshold_supported,

                        gtk::Image {
                            add_css_class: "charge-limit-info-icon",
                            set_icon_name: Some("ld-info-symbolic"),
                        },

                        gtk::Label {
                            add_css_class: "charge-limit-info-text",
                            set_label: &t!("dropdown-battery-charge-limit-not-supported"),
                        },
                    },
                }
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let device = &init.battery.device;

        let model = Self {
            percentage: device.percentage.get(),
            state: device.state.get(),
            time_to_empty: device.time_to_empty.get(),
            time_to_full: device.time_to_full.get(),
            energy_rate: device.energy_rate.get(),
            energy: device.energy.get(),
            energy_full: device.energy_full.get(),
            capacity: device.capacity.get(),
            warning_level: device.warning_level.get(),
            is_present: device.is_present.get(),
            charge_end_threshold: device.charge_end_threshold.get(),
            charge_threshold_supported: device.charge_threshold_supported.get(),
            charge_threshold_enabled: device.charge_threshold_enabled.get(),
            battery: init.battery.clone(),
        };

        watchers::spawn(&sender, &init.battery);

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            BatterySectionInput::ChargeLimitToggled(enabled) => {
                self.handle_charge_limit_toggled(enabled, &sender);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            BatterySectionCmd::BatteryStateChanged => {
                self.refresh_battery_state();
            }
        }
    }
}
