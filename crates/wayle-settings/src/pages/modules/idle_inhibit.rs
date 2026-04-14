//! Idle inhibit module settings.

use wayle_config::Config;

use crate::{
    editors::{number::number_u32, text::text},
    pages::{
        nav::LeafEntry,
        sections::bar_button::{
            BarButtonFields, actions_section, bar_display_section, colors_section,
        },
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.idle_inhibit;

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
        id: "idle-inhibit",
        i18n_key: "settings-nav-idle-inhibit",
        icon: "ld-coffee-symbolic",
        spec: page_spec(
            "settings-page-idle-inhibit",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        number_u32(&module.startup_duration),
                        text(&module.icon_inactive),
                        text(&module.icon_active),
                        text(&module.format),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
