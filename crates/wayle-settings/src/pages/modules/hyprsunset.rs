//! Hyprsunset module settings.

use crate::pages::nav::LeafEntry;
use crate::editors::{number::{number_u32}, text::{text}};
use crate::pages::spec::{SectionSpec, page_spec};
use crate::pages::sections::bar_button::{BarButtonFields, actions_section, bar_display_section, colors_section};
use wayle_config::Config;


pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.hyprsunset;

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
        id: "hyprsunset",
        i18n_key: "settings-nav-hyprsunset",
        icon: "ld-sun-symbolic",
        spec: page_spec(
            "settings-page-hyprsunset",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        text(&m.format),
                        number_u32(&m.temperature),
                        number_u32(&m.gamma),
                        text(&m.icon_off),
                        text(&m.icon_on),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
