//! Separator module settings.

use wayle_config::Config;

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.separator;

    LeafEntry {
        id: "separator",
        i18n_key: "settings-nav-separator",
        icon: "ld-minus-symbolic",
        spec: page_spec(
            "settings-page-separator",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![helpers::number_u32(&m.size), helpers::spacing(&m.length)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![helpers::color_value(&m.color)],
                },
            ],
        ),
    }
}
