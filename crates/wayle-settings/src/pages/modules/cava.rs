//! Cava module settings.

use wayle_config::{
    Config,
    schemas::modules::{CavaBarCount, CavaFramerate, FrequencyHz},
};

use crate::pages::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let m = &config.modules.cava;

    LeafEntry {
        id: "cava",
        i18n_key: "settings-nav-cava",
        icon: "ld-audio-lines-symbolic",
        spec: page_spec(
            "settings-page-cava",
            vec![
                SectionSpec {
                    title_key: "settings-section-general",
                    items: vec![
                        helpers::number_newtype(
                            &m.bars,
                            1.0,
                            256.0,
                            1.0,
                            0,
                            |v: &CavaBarCount| v.value() as f64,
                            |v| CavaBarCount::new(v as u16),
                        ),
                        helpers::number_newtype(
                            &m.framerate,
                            1.0,
                            360.0,
                            1.0,
                            0,
                            |v: &CavaFramerate| v.value() as f64,
                            |v| CavaFramerate::new(v as u32),
                        ),
                        helpers::toggle(&m.stereo),
                        helpers::normalized(&m.noise_reduction),
                        helpers::number_f64(&m.monstercat, 0.0, 10.0, 0.1, 1),
                        helpers::number_u32(&m.waves),
                        helpers::number_newtype(
                            &m.low_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |v| FrequencyHz::new(v as u32),
                        ),
                        helpers::number_newtype(
                            &m.high_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |v| FrequencyHz::new(v as u32),
                        ),
                        helpers::enum_select(&m.input),
                        helpers::text(&m.source),
                        helpers::enum_select(&m.style),
                        helpers::enum_select(&m.direction),
                        helpers::number_u32(&m.bar_width),
                        helpers::number_u32(&m.bar_gap),
                        helpers::spacing(&m.internal_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![helpers::toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        helpers::color_value(&m.color),
                        helpers::color_value(&m.button_bg_color),
                        helpers::color_value(&m.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        helpers::text_like(&m.left_click),
                        helpers::text_like(&m.right_click),
                        helpers::text_like(&m.middle_click),
                        helpers::text_like(&m.scroll_up),
                        helpers::text_like(&m.scroll_down),
                    ],
                },
            ],
        ),
    }
}
