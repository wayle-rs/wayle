//! Slider backed by DebouncedSlider. Throttled at 100ms during drag,
//! with a final commit on release.

mod row;
use std::sync::Arc;

use relm4::{
    gtk::{glib, prelude::*},
    prelude::*,
};
pub(crate) use row::*;
use wayle_config::ConfigProperty;
use wayle_widgets::primitives::slider::DebouncedSlider;

use super::spawn_property_watcher;

pub(crate) struct SliderControl<T: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<T>,
    slider: DebouncedSlider,
    to_slider: fn(&T) -> f64,
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
pub(crate) enum SliderMsg {
    Refresh,
}

impl<T> SimpleComponent for SliderControl<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    type Init = SliderInit<T>;
    type Input = SliderMsg;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .hexpand(false)
            .valign(gtk::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let current = (init.to_slider)(&init.property.get());

        let slider = DebouncedSlider::with_label(0.0);
        slider.set_cursor_from_name(Some("pointer"));
        slider.set_range(init.range_min, init.range_max);
        slider.set_formatter(init.format_label);
        slider.set_value(current);

        let prop = Arc::new(init.property.clone());
        let from_slider = init.from_slider;

        slider.connect_closure(
            "committed",
            false,
            glib::closure_local!(move |_slider: DebouncedSlider, value: f64| {
                prop.set(from_slider(value));
            }),
        );

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            let _ = input_sender.send(SliderMsg::Refresh);
        });

        root.append(&slider);

        let model = Self {
            property: init.property,
            slider,
            to_slider: init.to_slider,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SliderMsg::Refresh => {
                let value = (self.to_slider)(&self.property.get());
                self.slider.set_value(value);
            }
        }
    }
}
