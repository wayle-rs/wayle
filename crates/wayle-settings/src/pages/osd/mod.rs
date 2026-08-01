//! OSD settings page: appearance and automatic display of on-screen indicators.

use wayle_config::Config;

use crate::{
    editors::{
        enum_select::enum_select,
        number::{number_u32, spacing},
        text::text_like,
        toggle::toggle,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let osd = &config.osd;

    LeafEntry {
        id: "osd",
        i18n_key: "settings-nav-osd",
        icon: "ld-monitor-symbolic",
        spec: page_spec(
            "settings-page-osd",
            vec![
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![
                        toggle(&osd.enabled),
                        enum_select(&osd.position),
                        enum_select(&osd.layer),
                        number_u32(&osd.duration),
                        text_like(&osd.monitor),
                        spacing(&osd.margin),
                        toggle(&osd.border),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-behavior",
                    items: vec![
                        toggle(&osd.auto_speaker),
                        toggle(&osd.auto_microphone),
                        toggle(&osd.auto_brightness),
                        toggle(&osd.auto_toggles),
                    ],
                },
            ],
        ),
    }
}
