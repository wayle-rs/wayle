//! SpinButton wired to a typed numeric property via caller-provided conversion functions.

mod row;
use std::sync::Arc;

use relm4::{
    gtk,
    gtk::{glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::{number_f64, number_newtype, number_u8, number_u32, number_u64, spacing};
use wayle_config::ConfigProperty;

use super::spawn_property_watcher;

pub(crate) struct NumberControl<T: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<T>,
    spin: gtk::SpinButton,
    handler_id: SignalHandlerId,
    to_f64: fn(&T) -> f64,
}

pub(crate) struct NumberInit<T: Clone + Send + Sync + PartialEq + 'static> {
    pub(crate) property: ConfigProperty<T>,
    pub(crate) range_min: f64,
    pub(crate) range_max: f64,
    pub(crate) step: f64,
    pub(crate) digits: u32,
    pub(crate) to_f64: fn(&T) -> f64,
    pub(crate) from_f64: fn(f64) -> T,
}

#[derive(Debug)]
pub(crate) enum NumberMsg {
    Refresh,
}

impl<T> SimpleComponent for NumberControl<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    type Init = NumberInit<T>;
    type Input = NumberMsg;
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
        let current = (init.to_f64)(&init.property.get());

        let spin = gtk::SpinButton::builder()
            .digits(init.digits)
            .numeric(true)
            .build();

        spin.set_cursor_from_name(Some("pointer"));
        spin.set_range(init.range_min, init.range_max);
        spin.set_increments(init.step, init.step * 10.0);
        spin.set_value(current);

        let prop = Arc::new(init.property.clone());
        let from_f64 = init.from_f64;

        let handler_id = spin.connect_value_changed(move |spin| {
            prop.set(from_f64(spin.value()));
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            input_sender.send(NumberMsg::Refresh).is_ok()
        });

        root.append(&spin);

        let model = Self {
            property: init.property,
            spin,
            handler_id,
            to_f64: init.to_f64,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            NumberMsg::Refresh => {
                let value = (self.to_f64)(&self.property.get());

                self.spin.block_signal(&self.handler_id);
                self.spin.set_value(value);
                self.spin.unblock_signal(&self.handler_id);
            }
        }
    }
}
