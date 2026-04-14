//! General settings page: fonts, display, scale, and rounding.

use wayle_config::Config;

use crate::{
    editors::{enum_select::enum_select, font::font, slider::scale, toggle::toggle},
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let general = &config.general;
    let styling = &config.styling;

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
                    title_key: "settings-section-scale-rounding",
                    items: vec![scale(&styling.scale), enum_select(&styling.rounding)],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![toggle(&general.tearing_mode)],
                },
            ],
        ),
    }
}
