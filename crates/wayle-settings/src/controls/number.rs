//! SpinButton wired to a typed numeric property via caller-provided conversion functions.

use std::sync::Arc;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::ControlOutput;

pub(crate) struct NumberControl<T: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<T>,
}

pub(crate) struct NumberInit<T: Clone + Send + Sync + PartialEq + 'static> {
    pub property: ConfigProperty<T>,
    pub range_min: f64,
    pub range_max: f64,
    pub step: f64,
    pub digits: u32,
    pub to_f64: fn(&T) -> f64,
    pub from_f64: fn(f64) -> T,
}

#[derive(Debug)]
pub(crate) enum NumberMsg {}

impl<T> SimpleComponent for NumberControl<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    type Init = NumberInit<T>;
    type Input = NumberMsg;
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
        let current = (init.to_f64)(&init.property.get());

        let spin = gtk4::SpinButton::builder()
            .digits(init.digits)
            .numeric(true)
            .build();

        spin.set_range(init.range_min, init.range_max);
        spin.set_increments(init.step, init.step * 10.0);
        spin.set_value(current);

        let prop = Arc::new(init.property.clone());
        let from_f64 = init.from_f64;
        let output_sender = sender.output_sender().clone();

        spin.connect_value_changed(move |spin| {
            prop.set(from_f64(spin.value()));
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        root.append(&spin);

        let model = Self {
            property: init.property,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
