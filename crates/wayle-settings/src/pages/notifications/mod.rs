//! Notifications settings page: popup display, positioning, and filtering.

use wayle_config::Config;

use super::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
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
                        helpers::enum_select(&notif.popup_position),
                        helpers::number_u32(&notif.popup_max_visible),
                        helpers::enum_select(&notif.popup_stacking_order),
                        helpers::number_u32(&notif.popup_duration),
                        helpers::toggle(&notif.popup_hover_pause),
                        helpers::toggle(&notif.popup_shadow),
                        helpers::enum_select(&notif.popup_close_behavior),
                        helpers::enum_select(&notif.popup_urgency_bar),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-positioning",
                    items: vec![
                        helpers::text_like(&notif.popup_monitor),
                        helpers::spacing(&notif.popup_margin_x),
                        helpers::spacing(&notif.popup_margin_y),
                        helpers::spacing(&notif.popup_gap),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-filtering",
                    items: vec![
                        helpers::enum_select(&notif.icon_source),
                        helpers::toml_editor(
                            &notif.blocklist,
                            "blocklist",
                            &config.styling.palette.bg,
                        ),
                    ],
                },
            ],
        ),
    }
}
