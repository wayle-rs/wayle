//! System tray module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.systray;

    LeafEntry {
        id: "systray",
        i18n_key: "settings-nav-systray",
        icon: "ld-panel-top-symbolic",
        spec: page_spec(
            "settings-page-systray",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        helpers::scale(&m.icon_scale),
                        helpers::spacing(&m.item_gap),
                        helpers::spacing(&m.internal_padding),
                        helpers::toml_editor(&m.blacklist, "blacklist", &config.styling.palette.bg),
                        helpers::toml_editor(&m.overrides, "overrides", &config.styling.palette.bg),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![helpers::toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        helpers::color_value(&m.border_color),
                        helpers::color_value(&m.button_bg_color),
                    ],
                },
            ],
        ),
    }
}
