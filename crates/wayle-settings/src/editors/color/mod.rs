//! Color picker for HexColor config properties using GTK's ColorDialog.

mod row;
use relm4::{
    gtk::{gdk, glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::color;
use wayle_config::{ConfigProperty, schemas::styling::HexColor};

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct ColorControl {
    property: ConfigProperty<HexColor>,
    button: gtk::ColorDialogButton,
    handler_id: SignalHandlerId,
    _watcher: WatcherHandle,
}

#[derive(Debug)]
pub(crate) enum ColorMsg {
    Refresh,
}

impl SimpleComponent for ColorControl {
    type Init = ConfigProperty<HexColor>;
    type Input = ColorMsg;
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
        let dialog = gtk::ColorDialog::new();
        let button = gtk::ColorDialogButton::new(Some(dialog));
        button.set_cursor_from_name(Some("pointer"));

        let current = hex_to_rgba(&property.get());
        button.set_rgba(&current);

        let prop = property.clone();

        let handler_id = button.connect_rgba_notify(move |button: &gtk::ColorDialogButton| {
            let rgba = button.rgba();
            let hex = rgba_to_hex(&rgba);

            if let Ok(color) = HexColor::new(&hex) {
                prop.set(color);
            }
        });

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&property, move || {
            input_sender.send(ColorMsg::Refresh).is_ok()
        });

        root.append(&button);

        let model = Self {
            property,
            button,
            handler_id,
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ColorMsg::Refresh => {
                let rgba = hex_to_rgba(&self.property.get());

                self.button.block_signal(&self.handler_id);
                self.button.set_rgba(&rgba);
                self.button.unblock_signal(&self.handler_id);
            }
        }
    }
}

fn hex_to_rgba(hex: &HexColor) -> gdk::RGBA {
    gdk::RGBA::parse(hex.as_str()).unwrap_or(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
}

fn rgba_to_hex(rgba: &gdk::RGBA) -> String {
    let red = (rgba.red() * 255.0).round() as u8;
    let green = (rgba.green() * 255.0).round() as u8;
    let blue = (rgba.blue() * 255.0).round() as u8;
    let alpha = (rgba.alpha() * 255.0).round() as u8;

    if alpha == 255 {
        return format!("#{red:02x}{green:02x}{blue:02x}");
    }

    format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}")
}
