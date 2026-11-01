pub(crate) mod messages;

use gtk::{glib, prelude::*};
use relm4::{gtk, prelude::*};
use wayle_widgets::prelude::DebouncedSlider;

pub(super) use self::messages::{
    BrightnessDeviceInit, BrightnessDeviceItemMsg, BrightnessDeviceItemOutput,
};

pub(super) struct BrightnessDeviceItem {
    pub name: String,
    pub subtitle: Option<String>,
    pub icon: &'static str,
    slider: DebouncedSlider,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for BrightnessDeviceItem {
    type Init = BrightnessDeviceInit;
    type Input = BrightnessDeviceItemMsg;
    type Output = BrightnessDeviceItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            add_css_class: "brightness-device",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                add_css_class: "brightness-device-header",

                gtk::Box {
                    add_css_class: "brightness-device-icon",
                    set_valign: gtk::Align::Center,

                    gtk::Image {
                        add_css_class: "brightness-device-icon-img",
                        set_icon_name: Some(self.icon),
                    },
                },

                gtk::Box {
                    add_css_class: "brightness-device-info",
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        add_css_class: "brightness-device-name",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        #[watch]
                        set_label: &self.name,
                    },

                    gtk::Label {
                        add_css_class: "brightness-device-meta",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        #[watch]
                        set_visible: self.subtitle.is_some(),
                        #[watch]
                        set_label: self.subtitle.as_deref().unwrap_or_default(),
                    },
                },
            },

            gtk::Box {
                add_css_class: "brightness-slider-row",

                #[local_ref]
                slider_widget -> gtk::Box {},
            },
        }
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            subtitle: init.subtitle,
            icon: init.icon,
            slider: DebouncedSlider::with_label(init.percentage),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        _root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        if let Some(scale) = self.slider.scale() {
            scale.add_css_class("brightness-slider");
        }
        if let Some(label) = self.slider.label_widget() {
            label.add_css_class("brightness-slider-value");
        }

        let commit_sender = sender.input_sender().clone();
        self.slider.connect_closure(
            "committed",
            false,
            glib::closure_local!(move |_slider: DebouncedSlider, percentage: f64| {
                commit_sender.emit(BrightnessDeviceItemMsg::BrightnessCommitted(percentage));
            }),
        );

        let slider_widget = self.slider.upcast_ref::<gtk::Box>();
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            BrightnessDeviceItemMsg::SetBackendBrightness(percentage) => {
                self.slider.set_value(percentage);
            }
            BrightnessDeviceItemMsg::BrightnessCommitted(percentage) => {
                let _ = sender.output(BrightnessDeviceItemOutput::BrightnessChanged(
                    self.name.clone(),
                    percentage,
                ));
            }
        }
    }
}
