//! GTK Switch bound to a boolean config property.

use futures::StreamExt;
use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::ControlOutput;

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

        spawn_watcher(&property, &sender);

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

fn spawn_watcher(property: &ConfigProperty<bool>, sender: &ComponentSender<ToggleControl>) {
    let mut stream = property.watch();
    let input_sender = sender.input_sender().clone();

    gtk4::glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            let _ = input_sender.send(ToggleMsg::Refresh);
        }
    });
}
