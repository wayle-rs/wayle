//! Cava audio visualizer module configuration.

mod types;

use schemars::schema_for;
pub use types::{BarCount, CavaDirection, CavaInput, CavaStyle, Framerate, FrequencyHz};
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, NormalizedF64, Spacing},
};

/// Cava audio visualizer module configuration.
#[wayle_config(bar_container)]
pub struct CavaConfig {
    /// Number of frequency bars.
    #[i18n("settings-modules-cava-bars")]
    #[default(BarCount::DEFAULT)]
    pub bars: ConfigProperty<BarCount>,

    /// Visualization update rate in frames per second.
    #[i18n("settings-modules-cava-framerate")]
    #[default(Framerate::DEFAULT)]
    pub framerate: ConfigProperty<Framerate>,

    /// Stereo channel visualization (splits bars between left and right).
    #[i18n("settings-modules-cava-stereo")]
    #[default(false)]
    pub stereo: ConfigProperty<bool>,

    /// Noise reduction filter strength.
    #[serde(rename = "noise-reduction")]
    #[i18n("settings-modules-cava-noise-reduction")]
    #[default(NormalizedF64::new(0.65))]
    pub noise_reduction: ConfigProperty<NormalizedF64>,

    /// Monstercat-style smoothing across adjacent bars (0.0 = off).
    #[i18n("settings-modules-cava-monstercat")]
    #[default(0.0)]
    pub monstercat: ConfigProperty<f64>,

    /// Wave-style smoothing (0 = off).
    #[i18n("settings-modules-cava-waves")]
    #[default(0_u32)]
    pub waves: ConfigProperty<u32>,

    /// Low frequency cutoff in Hz.
    #[serde(rename = "low-cutoff")]
    #[i18n("settings-modules-cava-low-cutoff")]
    #[default(FrequencyHz::new(50))]
    pub low_cutoff: ConfigProperty<FrequencyHz>,

    /// High frequency cutoff in Hz.
    #[serde(rename = "high-cutoff")]
    #[i18n("settings-modules-cava-high-cutoff")]
    #[default(FrequencyHz::new(17000))]
    pub high_cutoff: ConfigProperty<FrequencyHz>,

    /// Audio capture backend.
    #[i18n("settings-modules-cava-input")]
    #[default(CavaInput::default())]
    pub input: ConfigProperty<CavaInput>,

    /// Audio source identifier ("auto" for automatic selection).
    #[i18n("settings-modules-cava-source")]
    #[default(String::from("auto"))]
    pub source: ConfigProperty<String>,

    /// Visualization rendering style.
    #[i18n("settings-modules-cava-style")]
    #[default(CavaStyle::default())]
    pub style: ConfigProperty<CavaStyle>,

    /// Bar growth direction.
    #[i18n("settings-modules-cava-direction")]
    #[default(CavaDirection::default())]
    pub direction: ConfigProperty<CavaDirection>,

    /// Bar color.
    #[i18n("settings-modules-cava-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub color: ConfigProperty<ColorValue>,

    /// Module background color.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-cava-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Width of each frequency bar in pixels.
    #[serde(rename = "bar-width")]
    #[i18n("settings-modules-cava-bar-width")]
    #[default(6)]
    pub bar_width: ConfigProperty<u32>,

    /// Gap between frequency bars in pixels.
    #[serde(rename = "bar-gap")]
    #[i18n("settings-modules-cava-bar-gap")]
    #[default(1)]
    pub bar_gap: ConfigProperty<u32>,

    /// Padding at the ends of the visualizer.
    #[serde(rename = "internal-padding")]
    #[i18n("settings-modules-cava-internal-padding")]
    #[default(Spacing::new(0.5))]
    pub internal_padding: ConfigProperty<Spacing>,

    /// Display border around the visualizer.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-cava-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-cava-border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-cava-left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-cava-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-cava-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-cava-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-cava-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for CavaConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("cava"),
            icon: String::from("󰝚"),
            description: String::from("Audio frequency visualizer"),
            behavior_configs: vec![(String::from("cava"), || schema_for!(CavaConfig))],
            styling_configs: vec![],
        }
    }
}
