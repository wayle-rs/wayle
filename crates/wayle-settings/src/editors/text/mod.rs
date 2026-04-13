//! Text entry for string-like config properties. Commits on Enter.

mod row;
use relm4::{
    gtk,
    gtk::{glib::SignalHandlerId, prelude::*},
    prelude::*,
};
pub(crate) use row::{text, text_like};
use wayle_config::{
    ClickAction, ConfigProperty,
    schemas::{modules::PopupMonitor, osd::OsdMonitor},
};

use super::{WatcherHandle, spawn_property_watcher};

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
        self.as_deref().unwrap_or_default().to_owned()
    }

    fn from_entry_text(text: &str) -> Self {
        if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        }
    }
}

macro_rules! impl_monitor_text_like {
    ($type:ty) => {
        impl TextLike for $type {
            fn to_entry_text(&self) -> String {
                match self {
                    Self::Primary => String::from("primary"),
                    Self::Connector(name) => name.clone(),
                }
            }

            fn from_entry_text(text: &str) -> Self {
                if text.eq_ignore_ascii_case("primary") || text.is_empty() {
                    Self::Primary
                } else {
                    Self::Connector(text.to_owned())
                }
            }
        }
    };
}

impl_monitor_text_like!(OsdMonitor);
impl_monitor_text_like!(PopupMonitor);

impl TextLike for ClickAction {
    fn to_entry_text(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Dropdown(name) => format!("dropdown:{name}"),
            Self::Shell(cmd) => cmd.clone(),
        }
    }

    fn from_entry_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::None;
        }

        match text.strip_prefix("dropdown:") {
            Some(name) => Self::Dropdown(name.to_owned()),
            None => Self::Shell(text.to_owned()),
        }
    }
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
