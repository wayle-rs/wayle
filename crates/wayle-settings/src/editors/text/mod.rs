//! Text entry for string-like config properties. Commits on Enter.

mod helpers;
mod row;

use relm4::{
    gtk,
    gtk::{glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::{text, text_like};
use wayle_config::ConfigProperty;

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) trait TextLike: Clone + Send + Sync + PartialEq + 'static {
    fn to_entry_text(&self) -> String;
    fn from_entry_text(text: &str) -> Self;
}

pub(crate) struct TextControl<T: TextLike> {
    property: ConfigProperty<T>,
    entry: gtk::Entry,
    activate_id: SignalHandlerId,
    changed_id: SignalHandlerId,
    _watcher: WatcherHandle,
}

pub(crate) struct TextInit<T: TextLike> {
    pub(crate) property: ConfigProperty<T>,
    pub(crate) dirty_badge: gtk::Label,
}

#[derive(Debug)]
pub(crate) enum TextMsg {
    Refresh,
}

impl<T: TextLike> SimpleComponent for TextControl<T> {
    type Init = TextInit<T>;
    type Input = TextMsg;
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
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let entry = gtk::Entry::builder()
            .text(init.property.get().to_entry_text())
            .valign(gtk::Align::Center)
            .build();
        entry.add_css_class("setting-text-entry");

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = entry.connect_changed(move |_entry| {
            dirty_badge.set_visible(true);
        });

        let prop = init.property.clone();
        let dirty_badge_commit = init.dirty_badge;

        let activate_id = entry.connect_activate(move |entry| {
            prop.set(T::from_entry_text(&entry.text()));
            dirty_badge_commit.set_visible(false);
        });

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&init.property, move || {
            input_sender.send(TextMsg::Refresh).is_ok()
        });

        root.append(&entry);

        let model = Self {
            property: init.property,
            entry,
            activate_id,
            changed_id,
            _watcher: watcher,
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
