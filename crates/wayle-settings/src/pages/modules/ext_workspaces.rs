//! Generic ext-workspace-v1 module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        enum_select::enum_select,
        number::{scale, spacing},
        text::text_like,
        toggle::toggle,
        toml_editor::toml_editor,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.ext_workspaces;

    LeafEntry {
        id: "ext-workspaces",
        i18n_key: "settings-nav-ext-workspaces",
        icon: "ld-grid-2x2-symbolic",
        spec: page_spec(
            "settings-page-ext-workspaces",
            vec![
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![enum_select(&module.display_mode), toggle(&module.show_hidden)],
                },
                SectionSpec {
                    title_key: "settings-section-sizing",
                    items: vec![
                        spacing(&module.workspace_padding),
                        scale(&module.icon_size),
                        scale(&module.label_size),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-urgent",
                    items: vec![toggle(&module.urgent_show)],
                },
                SectionSpec {
                    title_key: "settings-section-mappings",
                    items: vec![
                        toml_editor(
                            &module.workspace_map,
                            "workspace-map",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &module.workspace_ignore,
                            "workspace-ignore",
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
                        color_value(&module.inactive_color),
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
