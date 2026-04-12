//! TOML source editor for complex config values (maps, lists, thresholds).

mod row;
pub(crate) use row::*;

pub(crate) mod helpers;

use std::{env, fs, path::PathBuf, sync::Mutex};

use gtk4::prelude::*;
use helpers::{deserialize_with_key, serialize_with_key};
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use sourceview5::prelude::*;
use tracing::warn;
use wayle_config::{
    ConfigProperty,
    schemas::styling::{HexColor, PaletteConfig},
};
use wayle_i18n::t;

use super::{ControlOutput, spawn_property_watcher};

const SCHEME_ID: &str = "wayle";
const SCHEME_FILENAME: &str = "wayle.xml";

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
    pub min_lines: Option<u32>,
    pub palette_bg: ConfigProperty<HexColor>,
}

static SCHEME_DIR_REGISTERED: Mutex<bool> = Mutex::new(false);

fn scheme_dir() -> PathBuf {
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join("wayle-sourceview");
    }

    let user = env::var("USER").unwrap_or_else(|_| String::from("unknown"));
    env::temp_dir().join(format!("wayle-sourceview-{user}"))
}

pub(crate) fn update_wayle_scheme(palette: &PaletteConfig) {
    let dir = scheme_dir();

    if let Err(err) = fs::create_dir_all(&dir) {
        warn!(error = %err, "failed to create scheme directory");
        return;
    }

    if let Err(err) = fs::write(dir.join(SCHEME_FILENAME), build_scheme_xml(palette)) {
        warn!(error = %err, "failed to write scheme file");
        return;
    }

    let manager = sourceview5::StyleSchemeManager::default();

    let mut registered = SCHEME_DIR_REGISTERED
        .lock()
        .unwrap_or_else(|err| err.into_inner());

    if !*registered {
        let Some(dir_str) = dir.to_str() else {
            warn!(path = %dir.display(), "scheme directory path is not valid UTF-8");
            return;
        };
        manager.append_search_path(dir_str);
        *registered = true;
    }

    manager.force_rescan();
}

fn build_scheme_xml(palette: &PaletteConfig) -> String {
    let bg = palette.bg.get();
    let surface = palette.surface.get();
    let fg = palette.fg.get();
    let fg_muted = palette.fg_muted.get();
    let primary = palette.primary.get();
    let red = palette.red.get();
    let green = palette.green.get();
    let yellow = palette.yellow.get();
    let blue = palette.blue.get();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<style-scheme id="{SCHEME_ID}" name="Wayle" version="1.0">
  <style name="text" foreground="{fg}" background="{bg}"/>
  <style name="cursor" foreground="{primary}"/>
  <style name="selection" foreground="{fg}" background="{surface}"/>
  <style name="current-line" background="{surface}"/>
  <style name="line-numbers" foreground="{fg_muted}" background="{bg}"/>
  <style name="bracket-match" foreground="{primary}" bold="true"/>

  <style name="def:keyword" foreground="{blue}" bold="true"/>
  <style name="def:string" foreground="{green}"/>
  <style name="def:number" foreground="{primary}"/>
  <style name="def:boolean" foreground="{primary}"/>
  <style name="def:comment" foreground="{fg_muted}" italic="true"/>
  <style name="def:type" foreground="{yellow}"/>
  <style name="def:constant" foreground="{primary}"/>
  <style name="def:identifier" foreground="{fg}"/>
  <style name="def:special-char" foreground="{red}"/>
  <style name="def:heading" foreground="{blue}" bold="true"/>
</style-scheme>"#
    )
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
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
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

        let scrolled = gtk4::ScrolledWindow::builder()
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

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.palette_bg, move || {
            let _ = input_sender.send(TomlEditorMsg::ReapplyScheme);
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

            TomlEditorMsg::ReapplyScheme => {
                let buffer = self.buffer.clone();

                gtk4::glib::idle_add_local_once(move || {
                    let manager = sourceview5::StyleSchemeManager::default();

                    if let Some(scheme) = manager.scheme(SCHEME_ID) {
                        buffer.set_style_scheme(Some(&scheme));
                    }
                });
            }
        }
    }
}
