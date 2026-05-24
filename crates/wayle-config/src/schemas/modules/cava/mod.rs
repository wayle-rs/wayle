//! Cava audio visualizer module configuration.

mod types;

use schemars::schema_for;
pub use types::{BarCount, CavaDirection, CavaInput, CavaStyle, Framerate, FrequencyHz};
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, NormalizedF64, Spacing},
};

/// Audio frequency bars visualising the output stream.
#[wayle_config(bar_container, i18n_prefix = "settings-modules-cava")]
pub struct CavaConfig {
    /// Number of frequency bars.
    #[default(BarCount::DEFAULT)]
    pub bars: ConfigProperty<BarCount>,

    /// Visualization update rate in frames per second.
    #[default(Framerate::DEFAULT)]
    pub framerate: ConfigProperty<Framerate>,

    /// Stereo channel visualization (splits bars between left and right).
    #[default(false)]
    pub stereo: ConfigProperty<bool>,

    /// Noise reduction filter strength.
    #[serde(rename = "noise-reduction")]
    #[default(NormalizedF64::new(0.65))]
    pub noise_reduction: ConfigProperty<NormalizedF64>,

    /// Monstercat-style smoothing across adjacent bars (0.0 = off).
    #[default(0.0)]
    pub monstercat: ConfigProperty<f64>,

    /// Wave-style smoothing (0 = off).
    #[default(0_u32)]
    pub waves: ConfigProperty<u32>,

    /// Low frequency cutoff in Hz.
    #[serde(rename = "low-cutoff")]
    #[default(FrequencyHz::new(50))]
    pub low_cutoff: ConfigProperty<FrequencyHz>,

    /// High frequency cutoff in Hz.
    #[serde(rename = "high-cutoff")]
    #[default(FrequencyHz::new(17000))]
    pub high_cutoff: ConfigProperty<FrequencyHz>,

    /// Audio capture backend.
    #[default(CavaInput::default())]
    pub input: ConfigProperty<CavaInput>,

    /// Audio source identifier ("auto" for automatic selection).
    #[default(String::from("auto"))]
    pub source: ConfigProperty<String>,

    /// Visualization rendering style.
    #[default(CavaStyle::default())]
    pub style: ConfigProperty<CavaStyle>,

    /// Bar growth direction.
    #[default(CavaDirection::default())]
    pub direction: ConfigProperty<CavaDirection>,

    /// Bar color.
    #[default(ColorValue::Token(CssToken::Accent))]
    pub color: ConfigProperty<ColorValue>,

    /// Module background color.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Width of each frequency bar in pixels.
    #[serde(rename = "bar-width")]
    #[default(6)]
    pub bar_width: ConfigProperty<u32>,

    /// Gap between frequency bars in pixels.
    #[serde(rename = "bar-gap")]
    #[default(1)]
    pub bar_gap: ConfigProperty<u32>,

    /// Padding at the ends of the visualizer.
    #[serde(rename = "internal-padding")]
    #[default(Spacing::new(0.5))]
    pub internal_padding: ConfigProperty<Spacing>,

    /// Display border around the visualizer.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for CavaConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("cava"),
            schema: || schema_for!(CavaConfig),
            layout_id: Some(String::from("cava")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(CavaConfig);
