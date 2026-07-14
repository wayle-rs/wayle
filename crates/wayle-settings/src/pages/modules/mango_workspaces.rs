//! MangoWM tag switcher module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        enum_select::enum_select,
        number::{number_u8, scale, spacing},
        text::{text, text_like},
        toggle::toggle,
        toml_editor::toml_editor,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

#[allow(clippy::too_many_lines)]
pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.mango_workspaces;

    LeafEntry {
        id: "mango-workspaces",
        i18n_key: "settings-nav-mango-workspaces",
        icon: "ld-grid-2x2-symbolic",
        spec: page_spec(
            "settings-page-mango-workspaces",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![toggle(&module.hide_empty), number_u8(&module.min_tag_count)],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![enum_select(&module.display_mode), text(&module.divider)],
                },
                SectionSpec {
                    title_key: "settings-section-app-icons",
                    items: vec![
                        toggle(&module.app_icons_show),
                        toggle(&module.app_icons_dedupe),
                        text(&module.app_icons_fallback),
                        text(&module.app_icons_empty),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-sizing",
                    items: vec![
                        spacing(&module.tag_padding),
                        spacing(&module.icon_gap),
                        scale(&module.icon_size),
                        scale(&module.label_size),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-urgent",
                    items: vec![
                        toggle(&module.urgent_show),
                        enum_select(&module.urgent_mode),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-mappings",
                    items: vec![
                        toml_editor(&module.tag_map, "tag-map", &config.styling.palette.bg),
                        toml_editor(
                            &module.app_icon_map,
                            "app-icon-map",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        enum_select(&module.active_indicator),
                        color_value(&module.active_color),
                        color_value(&module.occupied_color),
                        color_value(&module.empty_color),
                        color_value(&module.container_bg_color),
                        color_value(&module.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&module.left_click),
                        text_like(&module.middle_click),
                        text_like(&module.right_click),
                        text_like(&module.scroll_up),
                        text_like(&module.scroll_down),
                    ],
                },
            ],
        ),
    }
}
