//! Weather module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, BarButtonFields, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.weather;

    let fields = BarButtonFields {
        icon_show: &m.icon_show,
        label_show: &m.label_show,
        label_max_length: &m.label_max_length,
        border_show: &m.border_show,
        icon_color: &m.icon_color,
        icon_bg_color: &m.icon_bg_color,
        label_color: &m.label_color,
        button_bg_color: &m.button_bg_color,
        border_color: &m.border_color,
        left_click: &m.left_click,
        right_click: &m.right_click,
        middle_click: &m.middle_click,
        scroll_up: &m.scroll_up,
        scroll_down: &m.scroll_down,
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
                        helpers::enum_select(&m.provider),
                        helpers::text(&m.location),
                        helpers::enum_select(&m.units),
                        helpers::text(&m.format),
                        helpers::enum_select(&m.time_format),
                        helpers::number_u32(&m.refresh_interval_seconds),
                        helpers::text(&m.icon_name),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-api-keys",
                    items: vec![
                        helpers::text_like(&m.visual_crossing_key),
                        helpers::text_like(&m.weatherapi_key),
                    ],
                },
                helpers::bar_display_section(&fields),
                helpers::colors_section(&fields),
                helpers::actions_section(&fields),
            ],
        ),
    }
}
