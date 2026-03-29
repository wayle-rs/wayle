use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// Storage module configuration.
#[wayle_config(bar_button)]
pub struct StorageConfig {
    /// Polling interval in milliseconds.
    ///
    /// Faster polling increases CPU usage.
    #[serde(rename = "poll-interval-ms")]
    #[default(30000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Mount point to monitor (e.g., `"/"`, `"/home"`).
    #[serde(rename = "mount-point")]
    #[default(String::from("/"))]
    pub mount_point: ConfigProperty<String>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - Disk usage as integer (0-100)
    /// - `{{ used_tib }}` - Used space in TiB
    /// - `{{ used_gib }}` - Used space in GiB
    /// - `{{ used_mib }}` - Used space in MiB
    /// - `{{ used_auto }}` - Used space with auto unit (e.g., "128.5 GiB")
    /// - `{{ total_tib }}` - Total space in TiB
    /// - `{{ total_gib }}` - Total space in GiB
    /// - `{{ total_mib }}` - Total space in MiB
    /// - `{{ total_auto }}` - Total space with auto unit
    /// - `{{ free_tib }}` - Free space in TiB
    /// - `{{ free_gib }}` - Free space in GiB
    /// - `{{ free_mib }}` - Free space in MiB
    /// - `{{ free_auto }}` - Free space with auto unit
    /// - `{{ filesystem }}` - Filesystem type (e.g., "ext4", "btrfs")
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
    /// - `"{{ used_auto }}/{{ total_auto }}"` - "128.5 GiB/512.0 GiB"
    /// - `"{{ free_gib }} GiB free"` - "383.5 GiB free"
    #[serde(rename = "format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-hard-drive-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
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
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
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

    /// Dynamic color thresholds based on disk usage percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `above` for high-value warnings (e.g., disk nearly full).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.storage.thresholds]]
    /// above = 70
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.storage.thresholds]]
    /// above = 90
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for StorageConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("storage"),
            icon: String::from("󰋊"),
            description: String::from("Disk usage for a mount point"),
            behavior_configs: vec![(String::from("storage"), || schema_for!(StorageConfig))],
            styling_configs: vec![],
        }
    }
}
