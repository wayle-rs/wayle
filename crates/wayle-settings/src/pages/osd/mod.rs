//! OSD settings page: display options for on-screen indicators.

use wayle_config::Config;

use super::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let osd = &config.osd;

    LeafEntry {
        id: "osd",
        i18n_key: "settings-nav-osd",
        icon: "ld-monitor-symbolic",
        spec: page_spec(
            "settings-page-osd",
            vec![SectionSpec {
                title_key: "settings-section-display",
                items: vec![
                    helpers::toggle(&osd.enabled),
                    helpers::enum_select(&osd.position),
                    helpers::number_u32(&osd.duration),
                    helpers::text_like(&osd.monitor),
                    helpers::spacing(&osd.margin),
                    helpers::toggle(&osd.border),
                ],
            }],
        ),
    }
}
