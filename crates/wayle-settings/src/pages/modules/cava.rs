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
    let module = &config.modules.cava;

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
                            &module.bars,
                            1.0,
                            256.0,
                            1.0,
                            0,
                            |v: &CavaBarCount| v.value() as f64,
                            |count| CavaBarCount::new(count as u16),
                        ),
                        number_newtype(
                            &module.framerate,
                            1.0,
                            360.0,
                            1.0,
                            0,
                            |v: &CavaFramerate| v.value() as f64,
                            |fps| CavaFramerate::new(fps as u32),
                        ),
                        toggle(&module.stereo),
                        normalized(&module.noise_reduction),
                        number_f64(&module.monstercat, 0.0, 10.0, 0.1, 1),
                        number_u32(&module.waves),
                        number_newtype(
                            &module.low_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |hz| FrequencyHz::new(hz as u32),
                        ),
                        number_newtype(
                            &module.high_cutoff,
                            1.0,
                            50000.0,
                            1.0,
                            0,
                            |v: &FrequencyHz| v.value() as f64,
                            |hz| FrequencyHz::new(hz as u32),
                        ),
                        enum_select(&module.input),
                        text(&module.source),
                        enum_select(&module.style),
                        enum_select(&module.direction),
                        number_u32(&module.bar_width),
                        number_u32(&module.bar_gap),
                        spacing(&module.internal_padding),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-bar-display",
                    items: vec![toggle(&module.border_show)],
                },
                SectionSpec {
                    title_key: "settings-section-colors",
                    items: vec![
                        color_value(&module.color),
                        color_value(&module.button_bg_color),
                        color_value(&module.border_color),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-actions",
                    items: vec![
                        text_like(&module.left_click),
                        text_like(&module.right_click),
                        text_like(&module.middle_click),
                        text_like(&module.scroll_up),
                        text_like(&module.scroll_down),
                    ],
                },
            ],
        ),
    }
}
