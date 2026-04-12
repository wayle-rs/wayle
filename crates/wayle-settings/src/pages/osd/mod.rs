//! OSD settings page: display options for on-screen indicators.

use crate::pages::nav::LeafEntry;
use crate::editors::{enum_select::{enum_select}, toggle::{toggle}, number::{number_u32, spacing}, text::{text_like}};
use crate::pages::spec::{SectionSpec, page_spec};
use wayle_config::Config;


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
                    toggle(&osd.enabled),
                    enum_select(&osd.position),
                    number_u32(&osd.duration),
                    text_like(&osd.monitor),
                    spacing(&osd.margin),
                    toggle(&osd.border),
                ],
            }],
        ),
    }
}
