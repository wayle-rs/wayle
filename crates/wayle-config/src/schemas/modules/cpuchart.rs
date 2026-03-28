use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    schemas::{
        barchart::BarDirection,
        styling::{ColorValue, CssToken, Spacing},
    },
};

/// Configuration for the CPU chart module.
#[wayle_config(bar_container)]
pub struct CpuChartConfig {
    /// Module background color.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Width of each bar in pixels.
    #[serde(rename = "bar-width")]
    #[default(6)]
    pub bar_width: ConfigProperty<u32>,

    /// Gap between bars in pixels.
    #[serde(rename = "bar-gap")]
    #[default(1)]
    pub bar_gap: ConfigProperty<u32>,

    /// Bar growth direction.
    #[default(BarDirection::Normal)]
    pub direction: ConfigProperty<BarDirection>,

    /// Bar color.
    #[default(ColorValue::Token(CssToken::Accent))]
    pub color: ConfigProperty<ColorValue>,

    /// Padding at the ends of the chart.
    #[serde(rename = "internal-padding")]
    #[default(Spacing::new(0.5))]
    pub internal_padding: ConfigProperty<Spacing>,

    /// Show border around the module.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Left click action.
    #[serde(rename = "left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Right click action.
    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Middle click action.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Scroll up action.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Scroll down action.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}
