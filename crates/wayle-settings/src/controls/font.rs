//! Font picker using GTK's FontDialogButton. Only the family name is saved.

use std::sync::Arc;

use gtk4::{pango::FontDescription, prelude::*};
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::ControlOutput;

pub(crate) struct FontControl {
    property: ConfigProperty<String>,
}

#[derive(Debug)]
pub(crate) enum FontMsg {}

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

        let prop = Arc::new(property.clone());
        let output_sender = sender.output_sender().clone();

        button.connect_font_desc_notify(move |btn: &gtk4::FontDialogButton| {
            let Some(font_desc) = btn.font_desc() else {
                return;
            };
            let Some(family) = font_desc.family() else {
                return;
            };

            prop.set(family.to_string());
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        root.append(&button);

        let model = Self { property };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
