//! Bar button settings: style, icons, labels, borders, and button groups.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let bar = &config.bar;

    LeafEntry {
        id: "bar-button",
        i18n_key: "settings-nav-bar-button",
        icon: "ld-square-symbolic",
        spec: page_spec(
            "settings-page-bar-button",
            vec![
                SectionSpec {
                    title_key: "settings-section-style",
                    items: vec![
                        helpers::enum_select(&bar.button_variant),
                        helpers::percentage(&bar.button_opacity),
                        helpers::percentage(&bar.button_bg_opacity),
                        helpers::enum_select(&bar.button_rounding),
                        helpers::scale(&bar.button_gap),
                        helpers::enum_select(&bar.button_icon_position),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-icons",
                    items: vec![
                        helpers::scale(&bar.button_icon_size),
                        helpers::scale(&bar.button_icon_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-labels",
                    items: vec![
                        helpers::scale(&bar.button_label_size),
                        helpers::enum_select(&bar.button_label_weight),
                        helpers::scale(&bar.button_label_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-border",
                    items: vec![
                        helpers::enum_select(&bar.button_border_location),
                        helpers::number_u8(&bar.button_border_width),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-group",
                    items: vec![
                        helpers::percentage(&bar.button_group_opacity),
                        helpers::enum_select(&bar.button_group_rounding),
                        helpers::spacing(&bar.button_group_padding),
                        helpers::spacing(&bar.button_group_module_gap),
                        helpers::color_value(&bar.button_group_background),
                        helpers::enum_select(&bar.button_group_border_location),
                        helpers::number_u8(&bar.button_group_border_width),
                        helpers::color_value(&bar.button_group_border_color),
                    ],
                },
            ],
        ),
    }
}
