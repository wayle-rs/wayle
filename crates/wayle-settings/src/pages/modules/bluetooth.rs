//! Bluetooth module settings.

use wayle_config::Config;

use crate::{
    editors::text::text,
    pages::{
        nav::LeafEntry,
        sections::bar_button::{
            BarButtonFields, actions_section, bar_display_section, colors_section,
        },
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.bluetooth;

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
        id: "bluetooth",
        i18n_key: "settings-nav-bluetooth",
        icon: "ld-bluetooth-symbolic",
        spec: page_spec(
            "settings-page-bluetooth",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        text(&m.disabled_icon),
                        text(&m.disconnected_icon),
                        text(&m.connected_icon),
                        text(&m.searching_icon),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
