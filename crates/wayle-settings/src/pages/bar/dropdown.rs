//! Bar dropdown settings: behavior toggles and appearance.

use wayle_config::Config;

use crate::{
    editors::{slider::percentage, toggle::toggle},
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
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
                        toggle(&bar.dropdown_shadow),
                        toggle(&bar.dropdown_autohide),
                        toggle(&bar.dropdown_freeze_label),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-appearance",
                    items: vec![percentage(&bar.dropdown_opacity)],
                },
            ],
        ),
    }
}
