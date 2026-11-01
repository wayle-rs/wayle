//! Hyprland workspaces module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        enum_select::enum_select,
        number::{number_u8, scale, spacing},
        text::text,
        toggle::toggle,
        toml_editor::toml_editor,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.hyprland_workspaces;

    LeafEntry {
        id: "hyprland-workspaces",
        i18n_key: "settings-nav-hyprland-workspaces",
        icon: "ld-grid-2x2-symbolic",
        spec: page_spec(
            "settings-page-hyprland-workspaces",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        number_u8(&module.min_workspace_count),
                        toggle(&module.monitor_specific),
                        toggle(&module.show_special),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![
                        enum_select(&module.display_mode),
                        toggle(&module.label_use_name),
                        enum_select(&module.numbering),
                        text(&module.divider),
                    ],
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
                        spacing(&module.icon_gap),
                        spacing(&module.workspace_padding),
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
                        toml_editor(
                            &module.workspace_map,
                            "workspace-map",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &module.app_icon_map,
                            "app-icon-map",
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
                        color_value(&module.occupied_color),
                        color_value(&module.empty_color),
                        color_value(&module.container_bg_color),
                        color_value(&module.border_color),
                    ],
                },
            ],
        ),
    }
}
