use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// GPU module configuration.
///
/// Uses GPU data provided by `wayle-sysinfo` (NVML-backed on NVIDIA systems).
#[wayle_config(bar_button)]
pub struct GpuConfig {
    /// Polling interval in milliseconds.
    ///
    /// Faster polling increases monitoring overhead.
    #[serde(rename = "poll-interval-ms")]
    #[default(2000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Format string for the label.
    ///
    /// ## Aggregate Placeholders
    ///
    /// - `{{ count }}` - Number of detected GPUs
    /// - `{{ active_count }}` - Number of GPUs currently reporting utilization
    /// - `{{ avg_percent }}` - Average GPU core utilization (0-100)
    /// - `{{ avg_mem_percent }}` - Average GPU memory utilization (0-100)
    /// - `{{ max_temp_c }}` - Maximum GPU temperature in Celsius (if available)
    /// - `{{ total_power_w }}` - Total GPU power draw in watts across devices
    /// - `{{ hottest_gpu_name }}` - Name of the hottest GPU
    ///
    /// ## Per-GPU Placeholders (first two GPUs)
    ///
    /// - `{{ gpu0_percent }}`, `{{ gpu1_percent }}`
    /// - `{{ gpu0_mem_percent }}`, `{{ gpu1_mem_percent }}`
    /// - `{{ gpu0_temp_c }}`, `{{ gpu1_temp_c }}`
    /// - `{{ gpu0_mem_used_gib }}`, `{{ gpu1_mem_used_gib }}`
    /// - `{{ gpu0_mem_total_gib }}`, `{{ gpu1_mem_total_gib }}`
    /// - `{{ gpu0_name }}`, `{{ gpu1_name }}`
    /// - `{{ gpu0_power_w }}`, `{{ gpu1_power_w }}`
    /// - `{{ gpu0_power_limit_w }}`, `{{ gpu1_power_limit_w }}`
    /// - `{{ gpu0_fan_percent }}`, `{{ gpu1_fan_percent }}`
    /// - `{{ gpu0_graphics_mhz }}`, `{{ gpu1_graphics_mhz }}`
    /// - `{{ gpu0_memory_mhz }}`, `{{ gpu1_memory_mhz }}`
    ///
    /// ## Examples
    ///
    /// - `"{{ avg_percent }}%"` - `"37%"`
    /// - `"{{ gpu0_percent }}% | {{ gpu1_percent }}%"` - `"52% | 11%"`
    /// - `"{{ avg_percent }}% VRAM {{ avg_mem_percent }}%"` - `"37% VRAM 42%"`
    #[serde(rename = "format")]
    #[default(String::from("{{ avg_percent }}%"))]
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

    /// Dynamic color thresholds based on average GPU usage percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot.
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
            icon: String::from("󰢮"),
            description: String::from("GPU usage, memory, and temperature"),
            behavior_configs: vec![(String::from("gpu"), || schema_for!(GpuConfig))],
            styling_configs: vec![],
        }
    }
}
