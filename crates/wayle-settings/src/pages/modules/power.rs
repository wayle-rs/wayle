//! Power module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

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
                    items: vec![helpers::text(&m.icon_name)],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![helpers::toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        helpers::color_value(&m.icon_color),
                        helpers::color_value(&m.icon_bg_color),
                        helpers::color_value(&m.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        helpers::text_like(&m.left_click),
                        helpers::text_like(&m.right_click),
                        helpers::text_like(&m.middle_click),
                        helpers::text_like(&m.scroll_up),
                        helpers::text_like(&m.scroll_down),
                    ],
                },
            ],
        ),
    }
}
