//! Hyprland workspaces module settings.

use crate::pages::nav::LeafEntry;
use crate::editors::{color_value::{color_value}, enum_select::{enum_select}, toggle::{toggle}, toml_editor::{toml_editor}, slider::{scale}, number::{number_u8, spacing}, text::{text}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.hyprland_workspaces;

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
                        number_u8(&m.min_workspace_count),
                        toggle(&m.monitor_specific),
                        toggle(&m.show_special),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![
                        enum_select(&m.display_mode),
                        toggle(&m.label_use_name),
                        enum_select(&m.numbering),
                        text(&m.divider),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-app-icons",
                    items: vec![
                        toggle(&m.app_icons_show),
                        toggle(&m.app_icons_dedupe),
                        text(&m.app_icons_fallback),
                        text(&m.app_icons_empty),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-sizing",
                    items: vec![
                        spacing(&m.icon_gap),
                        spacing(&m.workspace_padding),
                        scale(&m.icon_size),
                        scale(&m.label_size),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-urgent",
                    items: vec![
                        toggle(&m.urgent_show),
                        enum_select(&m.urgent_mode),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-mappings",
                    items: vec![
                        toml_editor(
                            &m.workspace_map,
                            "workspace-map",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &m.app_icon_map,
                            "app-icon-map",
                            &config.styling.palette.bg,
                        ),
                        toml_editor(
                            &m.workspace_ignore,
                            "workspace-ignore",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        enum_select(&m.active_indicator),
                        color_value(&m.active_color),
                        color_value(&m.occupied_color),
                        color_value(&m.empty_color),
                        color_value(&m.container_bg_color),
                        color_value(&m.border_color),
                    ],
                },
            ],
        ),
    }
}
