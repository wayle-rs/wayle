//! Page spec types and layout builders.

mod bar_button;
mod layout;
pub(crate) mod types;

pub(crate) use bar_button::{
    BarButtonFields, actions_section, bar_display_section, colors_section,
};
pub(crate) use layout::{build_page_header, build_sections};
pub(crate) use types::{Keepalive, PageSpec, SectionSpec, page_spec};

pub(crate) use crate::editors::{
    bar_layout::bar_layout, color::color, color_value::color_value, enum_select::enum_select,
    file_picker::file_path, font::font, monitor_wallpaper::monitor_wallpaper,
    slider::{normalized, percentage, scale, signed_normalized},
    number::{number_f64, number_newtype, number_u8, number_u32, number_u64, spacing},
    text::{text, text_like},
    theme_selector::theme_selector,
    toggle::toggle,
    toml_editor::{toml_editor, toml_editor_sized},
};
