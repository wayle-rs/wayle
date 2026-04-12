//! Notifications settings page: popup display, positioning, and filtering.

use wayle_config::Config;

use crate::{
    editors::{
        enum_select::enum_select,
        number::{number_u32, spacing},
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
    let notif = &config.modules.notification;

    LeafEntry {
        id: "notifications",
        i18n_key: "settings-nav-notifications",
        icon: "ld-bell-symbolic",
        spec: page_spec(
            "settings-page-notifications",
            vec![
                SectionSpec {
                    title_key: "settings-section-popup-display",
                    items: vec![
                        enum_select(&notif.popup_position),
                        number_u32(&notif.popup_max_visible),
                        enum_select(&notif.popup_stacking_order),
                        number_u32(&notif.popup_duration),
                        toggle(&notif.popup_hover_pause),
                        toggle(&notif.popup_shadow),
                        enum_select(&notif.popup_close_behavior),
                        enum_select(&notif.popup_urgency_bar),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-positioning",
                    items: vec![
                        text_like(&notif.popup_monitor),
                        spacing(&notif.popup_margin_x),
                        spacing(&notif.popup_margin_y),
                        spacing(&notif.popup_gap),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-filtering",
                    items: vec![
                        enum_select(&notif.icon_source),
                        toml_editor(&notif.blocklist, "blocklist", &config.styling.palette.bg),
                    ],
                },
            ],
        ),
    }
}
