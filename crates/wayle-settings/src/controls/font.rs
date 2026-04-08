//! Font picker using GTK's FontDialogButton. Only the family name is saved.

use gtk4::{pango::FontDescription, prelude::*};
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct FontControl {
    property: ConfigProperty<String>,
    button: gtk4::FontDialogButton,
    handler_id: gtk4::glib::SignalHandlerId,
}

#[derive(Debug)]
pub(crate) enum FontMsg {
    Refresh,
}

impl SimpleComponent for FontControl {
    type Init = ConfigProperty<String>;
    type Input = FontMsg;
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
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dialog = gtk4::FontDialog::new();
        let button = gtk4::FontDialogButton::new(Some(dialog));

        let current_font = FontDescription::from_string(&property.get());
        button.set_font_desc(&current_font);

        let prop = property.clone();
        let output_sender = sender.output_sender().clone();

        let handler_id = button.connect_font_desc_notify(move |btn: &gtk4::FontDialogButton| {
            let Some(font_desc) = btn.font_desc() else {
                return;
            };
            let Some(family) = font_desc.family() else {
                return;
            };

            prop.set(family.to_string());
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            let _ = input_sender.send(FontMsg::Refresh);
        });

        root.append(&button);

        let model = Self {
            property,
            button,
            handler_id,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            FontMsg::Refresh => {
                let font = FontDescription::from_string(&self.property.get());

                self.button.block_signal(&self.handler_id);
                self.button.set_font_desc(&font);
                self.button.unblock_signal(&self.handler_id);
            }
        }
    }
}
