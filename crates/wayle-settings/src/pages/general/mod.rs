//! General settings page: fonts and display options.

use crate::pages::nav::LeafEntry;
use crate::editors::{toggle::{toggle}, font::{font}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


pub(crate) fn entry(config: &Config) -> LeafEntry {
    let general = &config.general;

    LeafEntry {
        id: "general",
        i18n_key: "settings-nav-general",
        icon: "ld-settings-symbolic",
        spec: page_spec(
            "settings-page-general",
            vec![
                SectionSpec {
                    title_key: "settings-section-fonts",
                    items: vec![
                        font(&general.font_sans),
                        font(&general.font_mono),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![toggle(&general.tearing_mode)],
                },
            ],
        ),
    }
}
