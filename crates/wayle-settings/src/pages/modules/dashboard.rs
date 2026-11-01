//! Dashboard module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        text::{text, text_like},
        toggle::toggle,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.dashboard;

    LeafEntry {
        id: "dashboard",
        i18n_key: "settings-nav-dashboard",
        icon: "ld-layout-dashboard-symbolic",
        spec: page_spec(
            "settings-page-dashboard",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![text(&module.icon_override)],
                },
                SectionSpec {
                    title_key: "settings-section-commands",
                    items: vec![
                        text(&module.dropdown_lock_command),
                        text(&module.dropdown_logout_command),
                        text(&module.dropdown_reboot_command),
                        text(&module.dropdown_poweroff_command),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&module.icon_color),
                        color_value(&module.icon_bg_color),
                        color_value(&module.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&module.left_click),
                        text_like(&module.right_click),
                        text_like(&module.middle_click),
                        text_like(&module.scroll_up),
                        text_like(&module.scroll_down),
                    ],
                },
            ],
        ),
    }
}
