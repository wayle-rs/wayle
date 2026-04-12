//! Text entry for string-like config properties. Commits on Enter.

mod row;
pub(crate) use row::*;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ClickAction, ConfigProperty,
    schemas::{modules::PopupMonitor, osd::OsdMonitor},
};

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
