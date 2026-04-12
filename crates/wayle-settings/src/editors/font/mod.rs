//! Font picker using GTK's FontDialogButton. Only the family name is saved.

mod row;
use relm4::{
    gtk::{glib::SignalHandlerId, pango::FontDescription, prelude::*},
    prelude::*,
};
pub(crate) use row::font;
use wayle_config::ConfigProperty;

use super::spawn_property_watcher;

pub(crate) struct FontControl {
    property: ConfigProperty<String>,
    button: gtk::FontDialogButton,
    handler_id: SignalHandlerId,
}

#[derive(Debug)]
pub(crate) enum FontMsg {
    Refresh,
}

impl SimpleComponent for FontControl {
    type Init = ConfigProperty<String>;
    type Input = FontMsg;
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
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let dialog = gtk::FontDialog::new();
        let button = gtk::FontDialogButton::new(Some(dialog));
        button.set_cursor_from_name(Some("pointer"));

        let current_font = FontDescription::from_string(&property.get());
        button.set_font_desc(&current_font);

        let prop = property.clone();

        let handler_id = button.connect_font_desc_notify(move |btn: &gtk::FontDialogButton| {
            let Some(font_desc) = btn.font_desc() else {
                return;
            };
            let Some(family) = font_desc.family() else {
                return;
            };

            prop.set(family.to_string());
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            input_sender.send(FontMsg::Refresh).is_ok()
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
