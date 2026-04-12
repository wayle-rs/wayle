//! Power module settings.

use crate::pages::nav::LeafEntry;
use crate::editors::{color_value::{color_value}, toggle::{toggle}, text::{text_like, text}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.power;

    LeafEntry {
        id: "power",
        i18n_key: "settings-nav-power",
        icon: "ld-power-symbolic",
        spec: page_spec(
            "settings-page-power",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![text(&m.icon_name)],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&m.icon_color),
                        color_value(&m.icon_bg_color),
                        color_value(&m.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&m.left_click),
                        text_like(&m.right_click),
                        text_like(&m.middle_click),
                        text_like(&m.scroll_up),
                        text_like(&m.scroll_down),
                    ],
                },
            ],
        ),
    }
}
