//! Text entry for string-like config properties. Commits on Enter.

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) trait TextLike: Clone + Send + Sync + PartialEq + 'static {
    fn to_entry_text(&self) -> String;
    fn from_entry_text(text: &str) -> Self;
}

impl TextLike for String {
    fn to_entry_text(&self) -> String {
        self.clone()
    }

    fn from_entry_text(text: &str) -> Self {
        text.to_string()
    }
}

impl TextLike for Option<String> {
    fn to_entry_text(&self) -> String {
        self.clone().unwrap_or_default()
    }

    fn from_entry_text(text: &str) -> Self {
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }
}

pub(crate) struct TextControl<T: TextLike> {
    property: ConfigProperty<T>,
    entry: gtk4::Entry,
    activate_id: gtk4::glib::SignalHandlerId,
    changed_id: gtk4::glib::SignalHandlerId,
}

pub(crate) struct TextInit<T: TextLike> {
    pub property: ConfigProperty<T>,
    pub dirty_badge: gtk4::Label,
}

#[derive(Debug)]
pub(crate) enum TextMsg {
    Refresh,
}

impl<T: TextLike> SimpleComponent for TextControl<T> {
    type Init = TextInit<T>;
    type Input = TextMsg;
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
        let entry = gtk4::Entry::builder()
            .text(init.property.get().to_entry_text())
            .valign(gtk4::Align::Center)
            .build();
        entry.add_css_class("setting-text-entry");

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = entry.connect_changed(move |_entry| {
            dirty_badge.set_visible(true);
        });

        let prop = init.property.clone();
        let output_sender = sender.output_sender().clone();
        let dirty_badge_commit = init.dirty_badge;

        let activate_id = entry.connect_activate(move |entry| {
            prop.set(T::from_entry_text(&entry.text()));
            dirty_badge_commit.set_visible(false);
            let _ = output_sender.send(ControlOutput::ValueChanged);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            let _ = input_sender.send(TextMsg::Refresh);
        });

        root.append(&entry);

        let model = Self {
            property: init.property,
            entry,
            activate_id,
            changed_id,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            TextMsg::Refresh => {
                self.entry.block_signal(&self.activate_id);
                self.entry.block_signal(&self.changed_id);
                self.entry.set_text(&self.property.get().to_entry_text());
                self.entry.unblock_signal(&self.changed_id);
                self.entry.unblock_signal(&self.activate_id);
            }
        }
    }
}
