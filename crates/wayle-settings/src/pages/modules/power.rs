//! Power module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        text::{text, text_like},
        toggle::toggle,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.power;

    LeafEntry {
        id: "power",
        i18n_key: "settings-nav-power",
        icon: "ld-power-symbolic",
        spec: page_spec(
            "settings-page-power",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![text(&module.icon_name)],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&module.icon_color),
                        color_value(&module.icon_bg_color),
                        color_value(&module.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&module.left_click),
                        text_like(&module.right_click),
                        text_like(&module.middle_click),
                        text_like(&module.scroll_up),
                        text_like(&module.scroll_down),
                    ],
                },
            ],
        ),
    }
}
