use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    schemas::{
        styling::{ColorValue, CssToken, Spacing},
        types::chart::Direction,
    },
};

/// Configuration for the CPU chart module.
#[wayle_config(bar_button)]
pub struct CpuChartConfig {
    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-cpu-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color.
    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color.
    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

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
    #[default(Direction::Normal)]
    pub direction: ConfigProperty<Direction>,

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
