//! TOML source editor for complex config values (maps, lists, thresholds).

pub(crate) mod helpers;

use gtk4::prelude::*;
use helpers::{deserialize_with_key, serialize_with_key};
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use sourceview5::prelude::*;
use wayle_config::ConfigProperty;
use wayle_i18n::t;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct TomlEditorControl<
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
> {
    property: ConfigProperty<T>,
    buffer: sourceview5::Buffer,
    changed_id: gtk4::glib::SignalHandlerId,
    scrolled: gtk4::ScrolledWindow,
    dirty_badge: gtk4::Label,
    key: &'static str,
}

pub(crate) struct TomlEditorInit<T: Clone + Send + Sync + PartialEq + 'static> {
    pub property: ConfigProperty<T>,
    pub key: &'static str,
    pub dirty_badge: gtk4::Label,
}

#[derive(Debug)]
pub(crate) enum TomlEditorMsg {
    Save,
    Refresh,
}

impl<T> SimpleComponent for TomlEditorControl<T>
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    type Init = TomlEditorInit<T>;
    type Input = TomlEditorMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .hexpand(true)
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
        let scheme = scheme_manager.scheme("Adwaita-dark");

        let buffer = sourceview5::Buffer::new(None::<&gtk4::TextTagTable>);

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
        view.set_wrap_mode(gtk4::WrapMode::WordChar);

        let scrolled = gtk4::ScrolledWindow::builder().child(&view).build();
        scrolled.add_css_class("toml-editor");

        let dirty_badge = init.dirty_badge.clone();
        let changed_id = buffer.connect_changed(move |_buffer| {
            dirty_badge.set_visible(true);
        });

        let save_button = gtk4::Button::builder()
            .label(t("settings-apply"))
            .halign(gtk4::Align::End)
            .build();

        save_button.add_css_class("primary");
        save_button.set_cursor_from_name(Some("pointer"));

        let input_sender = sender.input_sender().clone();
        save_button.connect_clicked(move |_button| {
            let _ = input_sender.send(TomlEditorMsg::Save);
        });

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.property, move || {
            let _ = input_sender.send(TomlEditorMsg::Refresh);
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
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
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
                        let _ = sender.output(ControlOutput::ValueChanged);
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
        }
    }
}
