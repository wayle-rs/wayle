//! System tray module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        number::{scale, spacing},
        toggle::toggle,
        toml_editor::toml_editor,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let module = &config.modules.systray;

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
                        scale(&module.icon_scale),
                        spacing(&module.item_gap),
                        spacing(&module.internal_padding),
                        toml_editor(&module.blacklist, "blacklist", &config.styling.palette.bg),
                        toml_editor(&module.overrides, "overrides", &config.styling.palette.bg),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&module.border_color),
                        color_value(&module.button_bg_color),
                    ],
                },
            ],
        ),
    }
}
