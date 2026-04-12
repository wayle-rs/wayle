//! Separator module settings.

use wayle_config::Config;

use crate::{
    editors::{
        color_value::color_value,
        number::{number_u32, spacing},
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
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
                    items: vec![number_u32(&m.size), spacing(&m.length)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![color_value(&m.color)],
                },
            ],
        ),
    }
}
