//! Bar dropdown settings: behavior toggles and appearance.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let bar = &config.bar;

    LeafEntry {
        id: "bar-dropdown",
        i18n_key: "settings-nav-bar-dropdown",
        icon: "ld-panel-bottom-symbolic",
        spec: page_spec(
            "settings-page-bar-dropdown",
            vec![
                SectionSpec {
                    title_key: "settings-section-behavior",
                    items: vec![
                        helpers::toggle(&bar.dropdown_shadow),
                        helpers::toggle(&bar.dropdown_autohide),
                        helpers::toggle(&bar.dropdown_freeze_label),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-appearance",
                    items: vec![helpers::percentage(&bar.dropdown_opacity)],
                },
            ],
        ),
    }
}
