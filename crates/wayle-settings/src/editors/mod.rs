//! One control per config type: toggle, dropdown, slider, spin button, font picker,
//! text entry, color picker, file picker, and TOML editor.
//! Each owns its `ConfigProperty` and writes back on user interaction.

use futures::StreamExt;
use relm4::{gtk, gtk::prelude::*};
use wayle_config::ConfigProperty;
use wayle_i18n::t;

pub mod bar_layout;
pub mod color;
pub mod color_value;
pub mod enum_select;
pub mod file_picker;
pub mod font;
pub mod monitor_wallpaper;
pub mod number;
pub mod slider;
pub mod text;
pub mod theme_selector;
pub mod toggle;
pub mod toml_editor;

pub(super) fn spawn_property_watcher<T: Clone + Send + Sync + PartialEq + 'static>(
    property: &ConfigProperty<T>,
    callback: impl Fn() + 'static,
) {
    let mut stream = property.watch();

    gtk::glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            callback();
        }
    });
}

pub(crate) fn make_dirty_badge() -> gtk::Label {
    let badge = gtk::Label::new(Some(&t("settings-source-unsaved")));

    badge.add_css_class("badge-subtle");
    badge.add_css_class("warning");

    badge.set_visible(false);
    badge.set_valign(gtk::Align::Center);
    badge.set_halign(gtk::Align::Start);

    badge
}
