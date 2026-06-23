//! Comma-separated text entry for `Vec<String>` properties. Commits on Enter.

mod helpers;
mod row;

use relm4::{
    gtk,
    gtk::{glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::vec_string;
use wayle_config::ConfigProperty;

use super::{WatcherHandle, spawn_property_watcher};

pub(crate) struct VecStringControl {
    property: ConfigProperty<Vec<String>>,
    entry: gtk::Entry,
    #[allow(dead_code)]
    dirty_badge: gtk::Label,
    activate_id: SignalHandlerId,
    changed_id: SignalHandlerId,
    _watcher: WatcherHandle,
}

pub(crate) struct VecStringInit {
    pub(crate) property: ConfigProperty<Vec<String>>,
    pub(crate) dirty_badge: gtk::Label,
}

#[derive(Debug)]
pub(crate) enum VecStringMsg {
    Refresh,
}

impl SimpleComponent for VecStringControl {
    type Init = VecStringInit;
    type Input = VecStringMsg;
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
            .text(VecStringControl::to_entry_text(&init.property.get()))
            .valign(gtk::Align::Center)
            .build();
        entry.add_css_class("setting-text-entry");

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = entry.connect_changed(move |_entry| {
            dirty_badge.set_visible(true);
        });

        let prop = init.property.clone();
        let dirty_badge_commit = init.dirty_badge.clone();

        let activate_id = entry.connect_activate(move |entry| {
            let value = VecStringControl::from_entry_text(&entry.text());
            prop.set(value);
            dirty_badge_commit.set_visible(false);
        });

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&init.property, move || {
            input_sender.send(VecStringMsg::Refresh).is_ok()
        });

        root.append(&entry);

        let model = Self {
            property: init.property,
            entry,
            dirty_badge: init.dirty_badge,
            activate_id,
            changed_id,
            _watcher: watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            VecStringMsg::Refresh => {
                self.entry.block_signal(&self.activate_id);
                self.entry.block_signal(&self.changed_id);
                self.entry.set_text(&Self::to_entry_text(&self.property.get()));
                self.entry.unblock_signal(&self.changed_id);
                self.entry.unblock_signal(&self.activate_id);
            }
        }
    }
}

impl VecStringControl {
    fn to_entry_text(values: &[String]) -> String {
        values.join(", ")
    }

    fn from_entry_text(text: &str) -> Vec<String> {
        if text.is_empty() {
            return Vec::new();
        }
        text.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}
