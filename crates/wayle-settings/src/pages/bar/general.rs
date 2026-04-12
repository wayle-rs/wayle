//! Bar general settings: layout, appearance, spacing, and border.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let bar = &config.bar;

    LeafEntry {
        id: "bar-general",
        i18n_key: "settings-nav-bar-general",
        icon: "ld-layout-dashboard-symbolic",
        spec: page_spec(
            "settings-page-bar-general",
            vec![
                SectionSpec {
                    title_key: "settings-section-layout",
                    items: vec![
                        helpers::bar_layout(&bar.layout, &config.modules.custom),
                        helpers::enum_select(&bar.location),
                        helpers::scale(&bar.scale),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-appearance",
                    items: vec![
                        helpers::color_value(&bar.bg),
                        helpers::percentage(&bar.background_opacity),
                        helpers::enum_select(&bar.rounding),
                        helpers::enum_select(&bar.shadow),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-spacing",
                    items: vec![
                        helpers::spacing(&bar.inset_edge),
                        helpers::spacing(&bar.inset_ends),
                        helpers::spacing(&bar.padding),
                        helpers::spacing(&bar.padding_ends),
                        helpers::spacing(&bar.module_gap),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-border",
                    items: vec![
                        helpers::enum_select(&bar.border_location),
                        helpers::number_u8(&bar.border_width),
                        helpers::color_value(&bar.border_color),
                    ],
                },
            ],
        ),
    }
}
