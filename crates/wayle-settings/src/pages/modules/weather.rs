//! Weather module settings.

use wayle_config::Config;

use crate::{
    editors::{
        enum_select::enum_select,
        number::number_u32,
        text::{text, text_like},
    },
    pages::{
        nav::LeafEntry,
        sections::bar_button::{
            BarButtonFields, actions_section, bar_display_section, colors_section,
        },
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.weather;

    let fields = BarButtonFields {
        icon_show: &module.icon_show,
        label_show: &module.label_show,
        label_max_length: &module.label_max_length,
        border_show: &module.border_show,
        icon_color: &module.icon_color,
        icon_bg_color: &module.icon_bg_color,
        label_color: &module.label_color,
        button_bg_color: &module.button_bg_color,
        border_color: &module.border_color,
        left_click: &module.left_click,
        right_click: &module.right_click,
        middle_click: &module.middle_click,
        scroll_up: &module.scroll_up,
        scroll_down: &module.scroll_down,
    };

    LeafEntry {
        id: "weather",
        i18n_key: "settings-nav-weather",
        icon: "ld-cloud-sun-symbolic",
        spec: page_spec(
            "settings-page-weather",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        enum_select(&module.provider),
                        text(&module.location),
                        enum_select(&module.units),
                        text(&module.format),
                        enum_select(&module.time_format),
                        number_u32(&module.refresh_interval_seconds),
                        text(&module.icon_name),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-api-keys",
                    items: vec![
                        text_like(&module.visual_crossing_key),
                        text_like(&module.weatherapi_key),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
