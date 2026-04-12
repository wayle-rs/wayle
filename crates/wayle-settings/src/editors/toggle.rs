//! GTK Switch bound to a boolean config property.

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::{ControlOutput, spawn_property_watcher};

pub struct ToggleControl {
    property: ConfigProperty<bool>,
    active: bool,
}

#[derive(Debug)]
pub enum ToggleMsg {
    Toggled(bool),
    Refresh,
}

#[relm4::component(pub)]
impl SimpleComponent for ToggleControl {
    type Init = ConfigProperty<bool>;
    type Input = ToggleMsg;
    type Output = ControlOutput;

    view! {
        gtk4::Switch {
            set_hexpand: false,
            set_valign: gtk4::Align::Center,
            set_cursor_from_name: Some("pointer"),

            #[watch]
            #[block_signal(toggle_handler)]
            set_active: model.active,

            connect_state_set[sender] => move |switch, active| {
                let _ = sender.input_sender().send(ToggleMsg::Toggled(active));
                switch.set_state(active);
                gtk4::glib::Propagation::Stop
            } @toggle_handler,
        }
    }

    fn init(
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let active = property.get();

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            let _ = input_sender.send(ToggleMsg::Refresh);
        });

        let model = Self { property, active };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ToggleMsg::Toggled(active) => {
                self.active = active;
                self.property.set(active);
                let _ = sender.output(ControlOutput::ValueChanged);
            }

            ToggleMsg::Refresh => {
                self.active = self.property.get();
            }
        }
    }
}
