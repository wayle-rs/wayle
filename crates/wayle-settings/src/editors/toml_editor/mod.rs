//! TOML source editor for complex config values (maps, lists, thresholds).

pub(crate) mod helpers;
mod row;

use helpers::{deserialize_with_key, serialize_with_key};
use relm4::{
    gtk,
    gtk::{glib, prelude::*},
    prelude::*,
};
pub(crate) use row::{toml_editor, toml_editor_sized};
use serde::{Deserialize, Serialize};
use sourceview5::prelude::*;
use wayle_config::{ConfigProperty, schemas::styling::HexColor};
use wayle_i18n::t;

use super::{WatcherHandle, spawn_property_watcher};
use crate::app::sourceview_scheme::SCHEME_ID;

pub(crate) struct TomlEditorControl<
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
> {
    property: ConfigProperty<T>,
    buffer: sourceview5::Buffer,
    changed_id: glib::SignalHandlerId,
    scrolled: gtk::ScrolledWindow,
    dirty_badge: gtk::Label,
    key: &'static str,
    _property_watcher: WatcherHandle,
    _palette_watcher: WatcherHandle,
}

pub(crate) struct TomlEditorInit<T: Clone + Send + Sync + PartialEq + 'static> {
    pub(crate) property: ConfigProperty<T>,
    pub(crate) key: &'static str,
    pub(crate) dirty_badge: gtk::Label,
    pub(crate) min_lines: Option<u32>,
    pub(crate) palette_bg: ConfigProperty<HexColor>,
}

#[derive(Debug)]
pub(crate) enum TomlEditorMsg {
    Save,
    Refresh,
    ReapplyScheme,
}

impl<T> SimpleComponent for TomlEditorControl<T>
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type Init = TomlEditorInit<T>;
    type Input = TomlEditorMsg;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .vexpand(true)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let language_manager = sourceview5::LanguageManager::default();
        let language = language_manager.language("toml");

        let scheme_manager = sourceview5::StyleSchemeManager::default();
        let scheme = scheme_manager
            .scheme(SCHEME_ID)
            .or_else(|| scheme_manager.scheme("Adwaita-dark"));

        let buffer = sourceview5::Buffer::new(None::<&gtk::TextTagTable>);

        if let Some(lang) = &language {
            buffer.set_language(Some(lang));
        }

        if let Some(scheme) = &scheme {
            buffer.set_style_scheme(Some(scheme));
        }

        buffer.set_highlight_syntax(true);
        buffer.set_text(&serialize_with_key(&init.property.get(), init.key));

        let view = sourceview5::View::with_buffer(&buffer);
        view.set_monospace(true);
        view.set_show_line_numbers(true);
        view.set_tab_width(2);
        view.set_auto_indent(true);
        view.set_wrap_mode(gtk::WrapMode::WordChar);

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&view)
            .vexpand(true)
            .build();
        scrolled.add_css_class("toml-editor");

        if let Some(lines) = init.min_lines {
            scrolled.add_css_class(&format!("toml-editor-lines-{lines}"));
        }

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = buffer.connect_changed(move |_buffer| {
            dirty_badge.set_visible(true);
        });

        let save_button = gtk::Button::builder()
            .label(t("settings-apply"))
            .halign(gtk::Align::End)
            .build();

        save_button.add_css_class("primary");
        save_button.set_cursor_from_name(Some("pointer"));

        let input_sender = sender.input_sender().clone();
        save_button.connect_clicked(move |_button| {
            let _ = input_sender.send(TomlEditorMsg::Save);
        });

        let input_sender = sender.input_sender().clone();
        let property_watcher = spawn_property_watcher(&init.property, move || {
            input_sender.send(TomlEditorMsg::Refresh).is_ok()
        });

        let input_sender = sender.input_sender().clone();
        let palette_watcher = spawn_property_watcher(&init.palette_bg, move || {
            input_sender.send(TomlEditorMsg::ReapplyScheme).is_ok()
        });

        root.append(&scrolled);
        root.append(&save_button);

        let model = Self {
            property: init.property,
            buffer,
            changed_id,
            scrolled,
            dirty_badge: init.dirty_badge,
            key: init.key,
            _property_watcher: property_watcher,
            _palette_watcher: palette_watcher,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            TomlEditorMsg::Save => {
                let start = self.buffer.start_iter();
                let end = self.buffer.end_iter();
                let text = self.buffer.text(&start, &end, false);

                match deserialize_with_key::<T>(&text, self.key) {
                    Some(value) => {
                        self.scrolled.remove_css_class("error");
                        self.property.set(value);
                        self.dirty_badge.set_visible(false);
                    }
                    None => {
                        self.scrolled.add_css_class("error");
                    }
                }
            }

            TomlEditorMsg::Refresh => {
                self.buffer.block_signal(&self.changed_id);
                let toml_text = serialize_with_key(&self.property.get(), self.key);
                self.buffer.set_text(&toml_text);
                self.buffer.unblock_signal(&self.changed_id);
                self.dirty_badge.set_visible(false);
            }

            TomlEditorMsg::ReapplyScheme => {
                let buffer = self.buffer.clone();

                glib::idle_add_local_once(move || {
                    let manager = sourceview5::StyleSchemeManager::default();

                    if let Some(scheme) = manager.scheme(SCHEME_ID) {
                        buffer.set_style_scheme(Some(&scheme));
                    }
                });
            }
        }
    }
}
