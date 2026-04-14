//! Keyboard input module settings.

use wayle_config::Config;

use crate::{
    editors::{text::text, toml_editor::toml_editor},
    pages::{
        nav::LeafEntry,
        sections::bar_button::{
            BarButtonFields, actions_section, bar_display_section, colors_section,
        },
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.keyboard_input;

    let fields = BarButtonFields {
        icon_show: &module.icon_show,
        label_show: &module.label_show,
        label_max_length: &module.label_max_length,
        border_show: &module.border_show,
        icon_color: &module.icon_color,
        icon_bg_color: &module.icon_bg_color,
        label_color: &module.label_color,
        button_bg_color: &module.button_bg_color,
        border_color: &module.border_color,
        left_click: &module.left_click,
        right_click: &module.right_click,
        middle_click: &module.middle_click,
        scroll_up: &module.scroll_up,
        scroll_down: &module.scroll_down,
    };

    LeafEntry {
        id: "keyboard-input",
        i18n_key: "settings-nav-keyboard-input",
        icon: "ld-keyboard-symbolic",
        spec: page_spec(
            "settings-page-keyboard-input",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        text(&module.format),
                        text(&module.icon_name),
                        toml_editor(
                            &module.layout_alias_map,
                            "layout-alias-map",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                bar_display_section(&fields),
                colors_section(&fields),
                actions_section(&fields),
            ],
        ),
    }
}
