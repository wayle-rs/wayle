//! Font family picker. MenuButton opens a searchable popover of system fonts.

mod helpers;
mod methods;
mod picker;
mod row;

use relm4::{
    gtk,
    gtk::{pango, prelude::*},
    prelude::*,
};
pub(crate) use row::font;
use wayle_config::ConfigProperty;

use self::picker::FontPicker;
use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct FontControl {
    property: ConfigProperty<String>,
    label: gtk::Label,
    _picker: Controller<FontPicker>,
    _watcher: WatcherHandle,
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
        let label = gtk::Label::new(Some(&property.get()));
        label.add_css_class("font-picker-label");
        label.set_ellipsize(pango::EllipsizeMode::End);
        label.set_max_width_chars(20);

        let button = gtk::MenuButton::builder().child(&label).build();
        button.add_css_class("font");
        button.set_cursor_from_name(Some("pointer"));

        let picker = FontPicker::builder().launch(property.clone()).detach();
        button.set_popover(Some(picker.widget()));

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&property, move || {
            input_sender.send(FontMsg::Refresh).is_ok()
        });

        root.append(&button);

        let model = Self {
            property,
            label,
            _picker: picker,
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            FontMsg::Refresh => {
                self.label.set_text(&self.property.get());
            }
        }
    }
}
