use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// CPU module configuration.
#[wayle_config(bar_button)]
pub struct CpuConfig {
    /// Polling interval in milliseconds.
    ///
    /// Faster polling increases CPU usage.
    #[serde(rename = "poll-interval-ms")]
    #[default(2000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Temperature sensor label. Use `"auto"` for automatic detection,
    /// or specify a label (e.g., `"Tctl"`, `"Package id 0"`).
    /// Run `sensors` to see available labels.
    #[serde(rename = "temp-sensor")]
    #[default(String::from("auto"))]
    pub temp_sensor: ConfigProperty<String>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - CPU usage (0-100)
    /// - `{{ freq_ghz }}` - Frequency of the busiest core (highest usage)
    /// - `{{ avg_freq_ghz }}` - Average frequency across cores
    /// - `{{ max_freq_ghz }}` - Maximum frequency among cores
    /// - `{{ temp_c }}` - Temperature in Celsius (if available)
    /// - `{{ temp_f }}` - Temperature in Fahrenheit (if available)
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
    /// - `"{{ percent }}% @ {{ freq_ghz }}GHz"` - "45% @ 3.2GHz"
    /// - `"{{ percent }}% {{ temp_c }}C"` - "45% 62C"
    #[serde(rename = "format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-cpu-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color.
    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

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

    /// Dynamic color thresholds based on CPU usage percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `above` for high-value warnings (e.g., high CPU usage).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.cpu.thresholds]]
    /// above = 70
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.cpu.thresholds]]
    /// above = 90
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for CpuConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("cpu"),
            icon: String::from("󰻠"),
            description: String::from("CPU usage, frequency, and temperature"),
            behavior_configs: vec![(String::from("cpu"), || schema_for!(CpuConfig))],
            styling_configs: vec![],
        }
    }
}
