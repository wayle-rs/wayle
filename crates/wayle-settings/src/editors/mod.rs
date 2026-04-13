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

/// Handle whose `Drop` impl calls [`glib::JoinHandle::abort`] on the
/// underlying watcher future.
///
/// The watcher loop awaits `stream.next().await`, which only yields when
/// the property value changes. While the config is idle, the callback
/// never runs, so `callback() -> false` can't end the loop on its own.
/// Holding this handle keeps the future alive; dropping it aborts the future.
#[must_use = "discarding the handle aborts the watcher subscription"]
pub(crate) struct WatcherHandle(glib::JoinHandle<()>);

impl Drop for WatcherHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Invokes `callback` on every property change after the initial emission.
///
/// `ConfigProperty::watch` yields the current value on subscribe; we swallow
/// it so editors don't re-emit `Refresh` during init. Dropping the returned
/// [`WatcherHandle`] aborts the future.
pub(super) fn spawn_property_watcher<T: Clone + Send + Sync + PartialEq + 'static>(
    property: &ConfigProperty<T>,
    callback: impl Fn() -> bool + 'static,
) -> WatcherHandle {
    let mut stream = property.watch();

    let handle = glib::spawn_future_local(async move {
        stream.next().await;

        while stream.next().await.is_some() {
            if !callback() {
                break;
            }
        }
    });

    WatcherHandle(handle)
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
