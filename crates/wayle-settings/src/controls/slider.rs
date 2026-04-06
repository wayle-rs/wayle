//! Slider backed by DebouncedSlider. Throttled at 100ms during drag,
//! with a final commit on release.

use std::sync::Arc;

use gtk4::{glib, prelude::*};
use relm4::prelude::*;
use wayle_config::ConfigProperty;
use wayle_widgets::primitives::slider::DebouncedSlider;

use super::ControlOutput;

pub(crate) struct SliderControl<T: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<T>,
}

pub(crate) struct SliderInit<T: Clone + Send + Sync + PartialEq + 'static> {
    pub property: ConfigProperty<T>,
    pub range_min: f64,
    pub range_max: f64,
    pub to_slider: fn(&T) -> f64,
    pub from_slider: fn(f64) -> T,
    pub format_label: fn(f64) -> String,
}

#[derive(Debug)]
pub(crate) enum SliderMsg {}

impl<T> SimpleComponent for SliderControl<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    type Init = SliderInit<T>;
    type Input = SliderMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .hexpand(false)
            .valign(gtk4::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let current = (init.to_slider)(&init.property.get());

        let slider = DebouncedSlider::with_label(0.0);
        slider.set_range(init.range_min, init.range_max);
        slider.set_formatter(init.format_label);
        slider.set_value(current);

        let prop = Arc::new(init.property.clone());
        let from_slider = init.from_slider;
        let output_sender = sender.output_sender().clone();

        slider.connect_closure(
            "committed",
            false,
            glib::closure_local!(move |_slider: DebouncedSlider, value: f64| {
                prop.set(from_slider(value));
                let _ = output_sender.send(ControlOutput::ValueChanged);
            }),
        );

        root.append(&slider);

        let model = Self {
            property: init.property,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
