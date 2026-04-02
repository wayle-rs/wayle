mod types;

pub use types::{
    CyclingInterval, CyclingMode, FitMode, MonitorWallpaperConfig, TransitionDuration,
    TransitionFps, TransitionType,
};
use wayle_derive::wayle_config;

use crate::ConfigProperty;

/// Wallpaper management configuration.
#[wayle_config]
pub struct WallpaperConfig {
    /// Enable the awww wallpaper engine. Disable to use an external wallpaper
    /// tool while keeping color extraction and theming.
    #[serde(rename = "engine-enabled")]
    #[i18n("settings-wallpaper-engine-enabled")]
    #[default(true)]
    pub engine_enabled: ConfigProperty<bool>,

    /// Transition animation type.
    #[serde(rename = "transition-type")]
    #[i18n("settings-wallpaper-transition-type")]
    #[default(TransitionType::Simple)]
    pub transition_type: ConfigProperty<TransitionType>,

    /// Transition animation duration in seconds.
    #[serde(rename = "transition-duration")]
    #[i18n("settings-wallpaper-transition-duration")]
    #[default(TransitionDuration::DEFAULT)]
    pub transition_duration: ConfigProperty<TransitionDuration>,

    /// Transition animation frame rate.
    #[serde(rename = "transition-fps")]
    #[i18n("settings-wallpaper-transition-fps")]
    #[default(TransitionFps::DEFAULT)]
    pub transition_fps: ConfigProperty<TransitionFps>,

    /// Enable automatic wallpaper cycling.
    #[serde(rename = "cycling-enabled")]
    #[i18n("settings-wallpaper-cycling-enabled")]
    #[default(false)]
    pub cycling_enabled: ConfigProperty<bool>,

    /// Directory containing wallpaper images for cycling.
    #[serde(rename = "cycling-directory")]
    #[i18n("settings-wallpaper-cycling-directory")]
    #[default(String::new())]
    pub cycling_directory: ConfigProperty<String>,

    /// Wallpaper cycling order.
    #[serde(rename = "cycling-mode")]
    #[i18n("settings-wallpaper-cycling-mode")]
    #[default(CyclingMode::Sequential)]
    pub cycling_mode: ConfigProperty<CyclingMode>,

    /// Time between wallpaper changes in minutes.
    #[serde(rename = "cycling-interval-mins")]
    #[i18n("settings-wallpaper-cycling-interval-mins")]
    #[default(CyclingInterval::DEFAULT)]
    pub cycling_interval_mins: ConfigProperty<CyclingInterval>,

    /// Show the same cycling wallpaper on all monitors. Only affects shuffle
    /// mode since sequential already displays the same image.
    #[serde(rename = "cycling-same-image")]
    #[i18n("settings-wallpaper-cycling-same-image")]
    #[default(false)]
    pub cycling_same_image: ConfigProperty<bool>,

    /// Per-monitor wallpaper and fit mode settings.
    #[i18n("settings-wallpaper-monitors")]
    #[default(Vec::new())]
    pub monitors: ConfigProperty<Vec<MonitorWallpaperConfig>>,
}
