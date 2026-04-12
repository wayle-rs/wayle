//! Wallpaper settings page: engine, transitions, cycling, and per-monitor config.

use wayle_config::{
    Config,
    schemas::wallpaper::{CyclingInterval, TransitionDuration, TransitionFps},
};

use super::{
    helpers::{self, SectionSpec, page_spec},
    nav::LeafEntry,
};

pub(crate) fn entry(config: &Config) -> LeafEntry {
    let wp = &config.wallpaper;

    LeafEntry {
        id: "wallpaper",
        i18n_key: "settings-nav-wallpaper",
        icon: "ld-image-symbolic",
        spec: page_spec(
            "settings-page-wallpaper",
            vec![
                SectionSpec {
                    title_key: "settings-section-engine",
                    items: vec![helpers::toggle(&wp.engine_enabled)],
                },
                SectionSpec {
                    title_key: "settings-section-transition",
                    items: vec![
                        helpers::enum_select(&wp.transition_type),
                        helpers::number_newtype(
                            &wp.transition_duration,
                            0.0,
                            30.0,
                            0.1,
                            1,
                            |v: &TransitionDuration| v.value() as f64,
                            |v| TransitionDuration::new(v as f32),
                        ),
                        helpers::number_newtype(
                            &wp.transition_fps,
                            1.0,
                            360.0,
                            1.0,
                            0,
                            |v: &TransitionFps| v.value() as f64,
                            |v| TransitionFps::new(v as u32),
                        ),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-cycling",
                    items: vec![
                        helpers::toggle(&wp.cycling_enabled),
                        helpers::file_path(&wp.cycling_directory),
                        helpers::enum_select(&wp.cycling_mode),
                        helpers::number_newtype(
                            &wp.cycling_interval_mins,
                            1.0,
                            1440.0,
                            1.0,
                            0,
                            |v: &CyclingInterval| v.value() as f64,
                            |v| CyclingInterval::new(v as u64),
                        ),
                        helpers::toggle(&wp.cycling_same_image),
                    ],
                },
                SectionSpec {
                    title_key: "settings-section-display",
                    items: vec![helpers::monitor_wallpaper(&wp.monitors)],
                },
            ],
        ),
    }
}
