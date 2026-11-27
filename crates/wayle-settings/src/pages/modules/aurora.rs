//! Aurora module settings.

use wayle_config::Config;

use crate::{
    editors::{
        enum_select::enum_select,
        number::number_u32,
        text::{text, text_like},
    },
    pages::{
        nav::LeafEntry,
        sections::bar_button::{BarButtonFields, actions_section, bar_display_section, colors_section},
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.aurora;

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
        id: "aurora",
        i18n_key: "settings-nav-aurora",
        icon: "ld-aurora-symbolic",
        spec: page_spec(
            "settings-page-aurora",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        text(&module.location),
                        text(&module.format),
                        enum_select(&module.time_format),
                        number_u32(&module.refresh_interval_seconds),
                        text(&module.icon_name),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
