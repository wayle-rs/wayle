mod color;
mod enum_select;
mod file_picker;
mod font;
mod layout_editors;
mod number;
mod slider;
mod text;
mod theme_selector;
mod toggle;
mod toml_editor;

pub(crate) use color::{color, color_value};
pub(crate) use enum_select::enum_select;
pub(crate) use file_picker::file_path;
pub(crate) use font::font;
use gtk4::prelude::*;
pub(crate) use layout_editors::{bar_layout, monitor_wallpaper};
pub(crate) use number::{number_f64, number_newtype, number_u8, number_u32, number_u64, spacing};
pub(crate) use slider::{normalized, percentage, scale, signed_normalized};
pub(crate) use text::{text, text_like};
pub(crate) use theme_selector::theme_selector;
pub(crate) use toggle::toggle;
pub(crate) use toml_editor::{toml_editor, toml_editor_sized};
use wayle_i18n::t;

pub(super) fn make_dirty_badge() -> gtk4::Label {
    let badge = gtk4::Label::new(Some(&t("settings-source-unsaved")));

    badge.add_css_class("badge-subtle");
    badge.add_css_class("warning");

    badge.set_visible(false);
    badge.set_valign(gtk4::Align::Center);
    badge.set_halign(gtk4::Align::Start);

    badge
}
