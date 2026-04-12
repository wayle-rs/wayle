//! General settings page: fonts and display options.

use wayle_config::Config;

use crate::{
    editors::{font::font, toggle::toggle},
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

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
                    items: vec![font(&general.font_sans), font(&general.font_mono)],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![toggle(&general.tearing_mode)],
                },
            ],
        ),
    }
}
