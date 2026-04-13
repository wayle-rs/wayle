//! GTK Switch bound to a boolean config property.

mod row;
use relm4::{
    gtk,
    gtk::{glib, prelude::*},
    prelude::*,
};
pub(crate) use row::toggle;
use wayle_config::ConfigProperty;

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct ToggleControl {
    property: ConfigProperty<bool>,
    active: bool,
    _watcher: WatcherHandle,
}

#[derive(Debug)]
pub(crate) enum ToggleMsg {
    Toggled(bool),
    Refresh,
}

#[relm4::component(pub(crate))]
impl SimpleComponent for ToggleControl {
    type Init = ConfigProperty<bool>;
    type Input = ToggleMsg;
    type Output = ();

    view! {
        gtk::Switch {
            set_hexpand: false,
            set_valign: gtk::Align::Center,
            set_cursor_from_name: Some("pointer"),

            #[watch]
            #[block_signal(toggle_handler)]
            set_active: model.active,

            connect_state_set[sender] => move |switch, active| {
                let _ = sender.input_sender().send(ToggleMsg::Toggled(active));
                switch.set_state(active);
                glib::Propagation::Stop
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
        let watcher = spawn_property_watcher(&property, move || {
            input_sender.send(ToggleMsg::Refresh).is_ok()
        });

        let model = Self {
            property,
            active,
            _watcher: watcher,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ToggleMsg::Toggled(active) => {
                self.active = active;
                self.property.set(active);
            }

            ToggleMsg::Refresh => {
                self.active = self.property.get();
            }
        }
    }
}
