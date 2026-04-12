//! Dashboard module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.dashboard;

    LeafEntry {
        id: "dashboard",
        i18n_key: "settings-nav-dashboard",
        icon: "ld-layout-dashboard-symbolic",
        spec: page_spec(
            "settings-page-dashboard",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![helpers::text(&m.icon_override)],
                },
                SectionSpec {
                    title_key: "settings-section-commands",
                    items: vec![
                        helpers::text(&m.dropdown_lock_command),
                        helpers::text(&m.dropdown_logout_command),
                        helpers::text(&m.dropdown_reboot_command),
                        helpers::text(&m.dropdown_poweroff_command),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![helpers::toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        helpers::color_value(&m.icon_color),
                        helpers::color_value(&m.icon_bg_color),
                        helpers::color_value(&m.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        helpers::text_like(&m.left_click),
                        helpers::text_like(&m.right_click),
                        helpers::text_like(&m.middle_click),
                        helpers::text_like(&m.scroll_up),
                        helpers::text_like(&m.scroll_down),
                    ],
                },
            ],
        ),
    }
}
