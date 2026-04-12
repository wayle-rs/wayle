//! Network module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, BarButtonFields, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.network;

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
        id: "network",
        i18n_key: "settings-nav-network",
        icon: "ld-wifi-symbolic",
        spec: page_spec(
            "settings-page-network",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        helpers::text(&m.wifi_disabled_icon),
                        helpers::text(&m.wifi_acquiring_icon),
                        helpers::text(&m.wifi_offline_icon),
                        helpers::text(&m.wifi_connected_icon),
                        helpers::text(&m.wired_connected_icon),
                        helpers::text(&m.wired_acquiring_icon),
                        helpers::text(&m.wired_disconnected_icon),
                        helpers::toml_editor(
                            &m.wifi_signal_icons,
                            "wifi-signal-icons",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                helpers::bar_display_section(&fields),
                helpers::colors_section(&fields),
                helpers::actions_section(&fields),
            ],
        ),
    }
}
