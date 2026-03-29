use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// RAM module configuration.
#[wayle_config(bar_button)]
pub struct RamConfig {
    /// Polling interval in milliseconds.
    ///
    /// Faster polling increases CPU usage.
    #[serde(rename = "poll-interval-ms")]
    #[default(5000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Format string for the label.
    ///
    /// ## Memory Placeholders
    ///
    /// - `{{ percent }}` - Memory usage as integer (0-100)
    /// - `{{ used_gib }}` - Used memory in GiB (e.g., "7.2")
    /// - `{{ total_gib }}` - Total memory in GiB (e.g., "16.0")
    /// - `{{ available_gib }}` - Available memory in GiB (e.g., "8.8")
    ///
    /// ## Swap Placeholders
    ///
    /// - `{{ swap_percent }}` - Swap usage as integer (0-100)
    /// - `{{ swap_used_gib }}` - Used swap in GiB
    /// - `{{ swap_total_gib }}` - Total swap in GiB
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
    /// - `"{{ used_gib }}/{{ total_gib }} GiB"` - "7.2/16.0 GiB"
    /// - `"{{ percent }}% (Swap: {{ swap_percent }}%)"` - "45% (Swap: 12%)"
    #[serde(rename = "format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-memory-stick-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Green))]
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
    #[default(ColorValue::Token(CssToken::Green))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Green))]
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

    /// Dynamic color thresholds based on RAM usage percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `above` for high-value warnings (e.g., high memory usage).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.ram.thresholds]]
    /// above = 80
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.ram.thresholds]]
    /// above = 95
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for RamConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("ram"),
            icon: String::from("󰍛"),
            description: String::from("Memory and swap usage"),
            behavior_configs: vec![(String::from("ram"), || schema_for!(RamConfig))],
            styling_configs: vec![],
        }
    }
}
