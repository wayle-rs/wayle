//! Volume module settings.

use crate::pages::nav::LeafEntry;
use crate::editors::{enum_select::{enum_select}, toml_editor::{toml_editor}, text::{text}};
use crate::pages::spec::{SectionSpec, page_spec};
use crate::pages::sections::bar_button::{BarButtonFields, actions_section, bar_display_section, colors_section};
use wayle_config::Config;


pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.volume;

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
        id: "volume",
        i18n_key: "settings-nav-volume",
        icon: "ld-volume-2-symbolic",
        spec: page_spec(
            "settings-page-volume",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        text(&m.icon_muted),
                        text(&m.format),
                        toml_editor(
                            &m.level_icons,
                            "level-icons",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &m.thresholds,
                            "thresholds",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-dropdown",
                    items: vec![enum_select(&m.dropdown_app_icons)],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
