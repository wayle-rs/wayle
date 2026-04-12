//! One control per config type: toggle, dropdown, slider, spin button, font picker,
//! text entry, color picker, file picker, and TOML editor.
//! Each owns its `ConfigProperty` and writes back on user interaction.

pub(crate) mod bar_layout;
pub(crate) mod color;
pub(crate) mod color_value;
pub(crate) mod enum_select;
pub(crate) mod file_picker;
pub(crate) mod font;
pub(crate) mod monitor_wallpaper;
pub(crate) mod number;
pub(crate) mod slider;
pub(crate) mod text;
pub(crate) mod theme_selector;
pub(crate) mod toggle;
pub(crate) mod toml_editor;

use futures::StreamExt;
use relm4::{
    gtk,
    gtk::{glib, prelude::*},
};
use wayle_config::ConfigProperty;
use wayle_i18n::t;

/// Invokes `callback` on every property change after the initial emission.
///
/// `ConfigProperty::watch` sends a snapshot on subscribe; we swallow it so
/// editors don't re-emit `Refresh` during init. When `callback` returns
/// `false` the subscription is dropped, so the watch doesn't outlive the
/// owning component.
pub(super) fn spawn_property_watcher<T: Clone + Send + Sync + PartialEq + 'static>(
    property: &ConfigProperty<T>,
    callback: impl Fn() -> bool + 'static,
) {
    let mut stream = property.watch();

    glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            if !callback() {
                break;
            }
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
