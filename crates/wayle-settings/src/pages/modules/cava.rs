//! Cava module settings.

use wayle_config::{
    Config,
    schemas::modules::{CavaBarCount, CavaFramerate, FrequencyHz},
};

use crate::{
    editors::{
        color_value::color_value,
        enum_select::enum_select,
        number::{number_f64, number_newtype, number_u32, spacing},
        slider::normalized,
        text::{text, text_like},
        toggle::toggle,
    },
    pages::{
        nav::LeafEntry,
        spec::{SectionSpec, page_spec},
    },
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
                        number_newtype(
                            &m.bars,
                            1.0,
                            256.0,
                            1.0,
                            0,
                            |v: &CavaBarCount| v.value() as f64,
                            |count| CavaBarCount::new(count as u16),
                        ),
                        number_newtype(
                            &m.framerate,
                            1.0,
                            360.0,
                            1.0,
                            0,
                            |v: &CavaFramerate| v.value() as f64,
                            |fps| CavaFramerate::new(fps as u32),
                        ),
                        toggle(&m.stereo),
                        normalized(&m.noise_reduction),
                        number_f64(&m.monstercat, 0.0, 10.0, 0.1, 1),
                        number_u32(&m.waves),
                        number_newtype(
                            &m.low_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |hz| FrequencyHz::new(hz as u32),
                        ),
                        number_newtype(
                            &m.high_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |hz| FrequencyHz::new(hz as u32),
                        ),
                        enum_select(&m.input),
                        text(&m.source),
                        enum_select(&m.style),
                        enum_select(&m.direction),
                        number_u32(&m.bar_width),
                        number_u32(&m.bar_gap),
                        spacing(&m.internal_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&m.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&m.color),
                        color_value(&m.button_bg_color),
                        color_value(&m.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&m.left_click),
                        text_like(&m.right_click),
                        text_like(&m.middle_click),
                        text_like(&m.scroll_up),
                        text_like(&m.scroll_down),
                    ],
                },
            ],
        ),
    }
}
