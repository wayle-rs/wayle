//! Bar button settings: style, icons, labels, borders, and button groups.

use crate::pages::nav::LeafEntry;
use crate::editors::{color_value::{color_value}, enum_select::{enum_select}, slider::{scale, percentage}, number::{number_u8, spacing}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


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
                        enum_select(&bar.button_variant),
                        percentage(&bar.button_opacity),
                        percentage(&bar.button_bg_opacity),
                        enum_select(&bar.button_rounding),
                        scale(&bar.button_gap),
                        enum_select(&bar.button_icon_position),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-icons",
                    items: vec![
                        scale(&bar.button_icon_size),
                        scale(&bar.button_icon_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-labels",
                    items: vec![
                        scale(&bar.button_label_size),
                        enum_select(&bar.button_label_weight),
                        scale(&bar.button_label_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-border",
                    items: vec![
                        enum_select(&bar.button_border_location),
                        number_u8(&bar.button_border_width),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-group",
                    items: vec![
                        percentage(&bar.button_group_opacity),
                        enum_select(&bar.button_group_rounding),
                        spacing(&bar.button_group_padding),
                        spacing(&bar.button_group_module_gap),
                        color_value(&bar.button_group_background),
                        enum_select(&bar.button_group_border_location),
                        number_u8(&bar.button_group_border_width),
                        color_value(&bar.button_group_border_color),
                    ],
                },
            ],
        ),
    }
}
