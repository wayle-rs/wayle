//! SpinButton wired to a typed numeric property via caller-provided conversion functions.

use std::sync::Arc;

use futures::StreamExt;
use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::ControlOutput;

pub(crate) struct NumberControl<T: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<T>,
    spin: gtk4::SpinButton,
    handler_id: gtk4::glib::SignalHandlerId,
    to_f64: fn(&T) -> f64,
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
pub(crate) enum NumberMsg {
    Refresh,
}

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

        let handler_id = spin.connect_value_changed(move |spin| {
            prop.set(from_f64(spin.value()));
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        spawn_watcher(&init.property, &sender);

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

fn spawn_watcher<T: Clone + Send + Sync + PartialEq + 'static>(
    property: &ConfigProperty<T>,
    sender: &ComponentSender<NumberControl<T>>,
) {
    let mut stream = property.watch();
    let input_sender = sender.input_sender().clone();

    gtk4::glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            let _ = input_sender.send(NumberMsg::Refresh);
        }
    });
}
