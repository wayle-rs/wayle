//! System tray module settings.

use crate::pages::nav::LeafEntry;
use crate::editors::{color_value::{color_value}, toggle::{toggle}, toml_editor::{toml_editor}, slider::{scale}, number::{spacing}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


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
                        scale(&m.icon_scale),
                        spacing(&m.item_gap),
                        spacing(&m.internal_padding),
                        toml_editor(&m.blacklist, "blacklist", &config.styling.palette.bg),
                        toml_editor(&m.overrides, "overrides", &config.styling.palette.bg),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&m.border_color),
                        color_value(&m.button_bg_color),
                    ],
                },
            ],
        ),
    }
}
