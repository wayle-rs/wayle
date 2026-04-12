//! Battery module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, BarButtonFields, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.battery;

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
        id: "battery",
        i18n_key: "settings-nav-battery",
        icon: "ld-battery-full-symbolic",
        spec: page_spec(
            "settings-page-battery",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        helpers::text(&m.charging_icon),
                        helpers::text(&m.alert_icon),
                        helpers::text(&m.format),
                        helpers::toml_editor(
                            &m.level_icons,
                            "level-icons",
                            &config.styling.palette.bg,
                        ),
                        helpers::toml_editor(
                            &m.thresholds,
                            "thresholds",
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
