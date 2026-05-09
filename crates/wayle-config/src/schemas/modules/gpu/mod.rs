use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// GPU usage, temperature, and VRAM.
///
/// Supports NVIDIA (via NVML) and AMD (via sysfs/hwmon).
/// The module auto-detects the GPU vendor on startup.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-gpu")]
pub struct GpuConfig {
    /// Polling interval in milliseconds.
    #[serde(rename = "poll-interval-ms")]
    #[default(5000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// GPU vendor override.
    ///
    /// Use `"auto"` for automatic detection, or force a specific vendor:
    /// `"nvidia"`, `"amd"`.
    #[serde(rename = "vendor")]
    #[default(String::from("auto"))]
    pub vendor: ConfigProperty<String>,

    /// GPU device index.
    ///
    /// For AMD, selects which `/sys/class/drm/card<N>` to read from.
    /// For NVIDIA, selects the NVML device index.
    /// Set to `0` for the first GPU, `1` for the second, etc.
    #[serde(rename = "card-index")]
    #[default(0)]
    pub card_index: ConfigProperty<u32>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - GPU usage (0-100)
    /// - `{{ temp_c }}` - Temperature in Celsius
    /// - `{{ temp_f }}` - Temperature in Fahrenheit
    /// - `{{ vram_used_mb }}` - VRAM used in MB
    /// - `{{ vram_total_mb }}` - VRAM total in MB
    /// - `{{ vram_percent }}` - VRAM usage percentage
    /// - `{{ name }}` - GPU model name
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
    /// - `"{{ percent }}% {{ temp_c }}°C"` - "45% 62°C"
    /// - `"{{ vram_used_mb }}/{{ vram_total_mb }}MB"` - "2048/8192MB"
    #[serde(rename = "format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-gpu-symbolic"))]
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

    /// Dynamic color thresholds based on GPU usage percentage.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.gpu.thresholds]]
    /// above = 70
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.gpu.thresholds]]
    /// above = 90
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for GpuConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("gpu"),
            schema: || schema_for!(GpuConfig),
            layout_id: Some(String::from("gpu")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(GpuConfig);
