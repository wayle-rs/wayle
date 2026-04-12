//! Shared section builders for modules that use the standard bar_button pattern.

use wayle_config::{ClickAction, ConfigProperty, schemas::styling::ColorValue};

use crate::{
    editors::{color_value::color_value, number::number_u32, text::text_like, toggle::toggle},
    pages::spec::SectionSpec,
};

pub(crate) struct BarButtonFields<'a> {
    pub icon_show: &'a ConfigProperty<bool>,
    pub label_show: &'a ConfigProperty<bool>,
    pub label_max_length: &'a ConfigProperty<u32>,
    pub border_show: &'a ConfigProperty<bool>,
    pub icon_color: &'a ConfigProperty<ColorValue>,
    pub icon_bg_color: &'a ConfigProperty<ColorValue>,
    pub label_color: &'a ConfigProperty<ColorValue>,
    pub button_bg_color: &'a ConfigProperty<ColorValue>,
    pub border_color: &'a ConfigProperty<ColorValue>,
    pub left_click: &'a ConfigProperty<ClickAction>,
    pub right_click: &'a ConfigProperty<ClickAction>,
    pub middle_click: &'a ConfigProperty<ClickAction>,
    pub scroll_up: &'a ConfigProperty<ClickAction>,
    pub scroll_down: &'a ConfigProperty<ClickAction>,
}

pub(crate) fn bar_display_section(fields: &BarButtonFields) -> SectionSpec {
    SectionSpec {
        title_key: "settings-section-bar-display",
        items: vec![
            toggle(fields.icon_show),
            toggle(fields.label_show),
            number_u32(fields.label_max_length),
            toggle(fields.border_show),
        ],
    }
}

pub(crate) fn colors_section(fields: &BarButtonFields) -> SectionSpec {
    SectionSpec {
        title_key: "settings-section-colors",
        items: vec![
            color_value(fields.icon_color),
            color_value(fields.icon_bg_color),
            color_value(fields.label_color),
            color_value(fields.button_bg_color),
            color_value(fields.border_color),
        ],
    }
}

pub(crate) fn actions_section(fields: &BarButtonFields) -> SectionSpec {
    SectionSpec {
        title_key: "settings-section-actions",
        items: vec![
            text_like(fields.left_click),
            text_like(fields.right_click),
            text_like(fields.middle_click),
            text_like(fields.scroll_up),
            text_like(fields.scroll_down),
        ],
    }
}
