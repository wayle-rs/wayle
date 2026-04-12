//! General settings page: fonts and display options.

use wayle_config::Config;

use super::{
    helpers::{self, SectionSpec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let general = &config.general;

    LeafEntry {
        id: "general",
        i18n_key: "settings-nav-general",
        icon: "ld-settings-symbolic",
        spec: helpers::page_spec(
            "settings-page-general",
            vec![
                SectionSpec {
                    title_key: "settings-section-fonts",
                    items: vec![
                        helpers::font(&general.font_sans),
                        helpers::font(&general.font_mono),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![helpers::toggle(&general.tearing_mode)],
                },
            ],
        ),
    }
}
