use wayle_config::schemas::wallpaper::{
    CyclingMode as CfgCyclingMode, FitMode as CfgFitMode, MonitorWallpaperConfig,
    TransitionType as CfgTransitionType,
};
use wayle_core::monitor::{MonitorInfo, best_match};
use wayle_wallpaper::{CyclingMode, FitMode, TransitionType, WallpaperService};

/// The `[[wallpaper.monitors]]` entry that best matches `connector`.
pub(crate) fn config_for<'a>(
    service: &WallpaperService,
    configs: &'a [MonitorWallpaperConfig],
    connector: &str,
) -> Option<&'a MonitorWallpaperConfig> {
    let monitor = service
        .outputs()
        .and_then(|outputs| outputs.find(connector))
        .unwrap_or_else(|| MonitorInfo::new(connector, "", "", None));

    best_match(configs, |cfg| cfg.name.as_str(), &monitor)
}

pub(crate) fn transition_type(cfg: CfgTransitionType) -> TransitionType {
    match cfg {
        CfgTransitionType::None => TransitionType::None,
        CfgTransitionType::Simple => TransitionType::Simple,
        CfgTransitionType::Fade => TransitionType::Fade {
            bezier: Default::default(),
        },
        CfgTransitionType::Left => TransitionType::Left,
        CfgTransitionType::Right => TransitionType::Right,
        CfgTransitionType::Top => TransitionType::Top,
        CfgTransitionType::Bottom => TransitionType::Bottom,
        CfgTransitionType::Wipe => TransitionType::Wipe {
            angle: Default::default(),
        },
        CfgTransitionType::Wave => TransitionType::Wave {
            angle: Default::default(),
            dimensions: Default::default(),
        },
        CfgTransitionType::Grow => TransitionType::Grow {
            position: Default::default(),
        },
        CfgTransitionType::Center => TransitionType::Center,
        CfgTransitionType::Outer => TransitionType::Outer {
            position: Default::default(),
        },
        CfgTransitionType::Any => TransitionType::Any,
        CfgTransitionType::Random => TransitionType::Random,
    }
}

pub(crate) fn fit_mode(cfg: CfgFitMode) -> FitMode {
    match cfg {
        CfgFitMode::Fill => FitMode::Fill,
        CfgFitMode::Fit => FitMode::Fit,
        CfgFitMode::Center => FitMode::Center,
        CfgFitMode::Stretch => FitMode::Stretch,
    }
}

pub(crate) fn cycling_mode(cfg: CfgCyclingMode) -> CyclingMode {
    match cfg {
        CfgCyclingMode::Sequential => CyclingMode::Sequential,
        CfgCyclingMode::Shuffle => CyclingMode::Shuffle,
    }
}
