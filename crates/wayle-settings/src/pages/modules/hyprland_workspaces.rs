//! Hyprland workspaces module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

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
                        helpers::number_u8(&m.min_workspace_count),
                        helpers::toggle(&m.monitor_specific),
                        helpers::toggle(&m.show_special),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![
                        helpers::enum_select(&m.display_mode),
                        helpers::toggle(&m.label_use_name),
                        helpers::enum_select(&m.numbering),
                        helpers::text(&m.divider),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-app-icons",
                    items: vec![
                        helpers::toggle(&m.app_icons_show),
                        helpers::toggle(&m.app_icons_dedupe),
                        helpers::text(&m.app_icons_fallback),
                        helpers::text(&m.app_icons_empty),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-sizing",
                    items: vec![
                        helpers::spacing(&m.icon_gap),
                        helpers::spacing(&m.workspace_padding),
                        helpers::scale(&m.icon_size),
                        helpers::scale(&m.label_size),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-urgent",
                    items: vec![
                        helpers::toggle(&m.urgent_show),
                        helpers::enum_select(&m.urgent_mode),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-mappings",
                    items: vec![
                        helpers::toml_editor(
                            &m.workspace_map,
                            "workspace-map",
                            &config.styling.palette.bg,
                        ),
                        helpers::toml_editor(
                            &m.app_icon_map,
                            "app-icon-map",
                            &config.styling.palette.bg,
                        ),
                        helpers::toml_editor(
                            &m.workspace_ignore,
                            "workspace-ignore",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![helpers::toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        helpers::enum_select(&m.active_indicator),
                        helpers::color_value(&m.active_color),
                        helpers::color_value(&m.occupied_color),
                        helpers::color_value(&m.empty_color),
                        helpers::color_value(&m.container_bg_color),
                        helpers::color_value(&m.border_color),
                    ],
                },
            ],
        ),
    }
}
