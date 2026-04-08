//! Bar dropdown settings: behavior toggles and appearance.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::ChildEntry,
};

pub(crate) fn entry(config: &Config) -> ChildEntry {
    let bar = &config.bar;

    ChildEntry {
        id: "bar-dropdown",
        i18n_key: "settings-nav-bar-dropdown",
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
