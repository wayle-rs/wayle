//! Page spec types, control factories, and layout builders.

mod bar_button;
pub(crate) mod controls;
mod layout;
mod types;

pub(crate) use bar_button::{
    BarButtonFields, actions_section, bar_display_section, colors_section,
};
pub(crate) use controls::{
    bar_layout, color, color_value, enum_select, file_path, font, monitor_wallpaper, normalized,
    number_f64, number_newtype, number_u8, number_u32, number_u64, percentage, scale,
    signed_normalized, spacing, text, text_like, theme_selector, toggle, toml_editor,
    toml_editor_sized,
};
pub(crate) use layout::{build_page_header, build_sections};
pub(crate) use types::{Keepalive, PageSpec, SectionSpec, page_spec};
