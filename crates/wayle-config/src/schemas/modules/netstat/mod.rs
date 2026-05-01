use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Network traffic counters (up/down rates).
#[wayle_config(bar_button, i18n_prefix = "settings-modules-netstat")]
pub struct NetstatConfig {
    /// Polling interval in milliseconds.
    ///
    /// Faster polling increases CPU usage.
    #[serde(rename = "poll-interval-ms")]
    #[default(2000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Network interface to monitor.
    ///
    /// Use `"auto"` to select the first active interface, or specify an
    /// interface name like `"eth0"` or `"wlan0"`.
    #[serde(rename = "interface")]
    #[default(String::from("auto"))]
    pub interface: ConfigProperty<String>,

    /// Format string for the label.
    ///
    /// ## Download Placeholders
    ///
    /// - `{{ down_kib }}` - Download speed in KiB/s
    /// - `{{ down_mib }}` - Download speed in MiB/s
    /// - `{{ down_gib }}` - Download speed in GiB/s
    /// - `{{ down_auto }}` - Download speed with auto unit (e.g., "1.5 MiB/s")
    ///
    /// ## Upload Placeholders
    ///
    /// - `{{ up_kib }}` - Upload speed in KiB/s
    /// - `{{ up_mib }}` - Upload speed in MiB/s
    /// - `{{ up_gib }}` - Upload speed in GiB/s
    /// - `{{ up_auto }}` - Upload speed with auto unit (e.g., "256 KiB/s")
    ///
    /// ## Other Placeholders
    ///
    /// - `{{ interface }}` - Interface name (e.g., "wlan0")
    ///
    /// ## Examples
    ///
    /// - `"{{ down_auto }} {{ up_auto }}"` - "1.5 MiB/s 256 KiB/s"
    /// - `"D:{{ down_mib }} U:{{ up_mib }}"` - "D:1.5 U:0.2"
    /// - `"{{ interface }}: {{ down_auto }}"` - "wlan0: 1.5 MiB/s"
    #[serde(rename = "format")]
    #[default(String::from("{{ down_auto }} {{ up_auto }}"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-activity-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Red))]
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
    #[default(ColorValue::Token(CssToken::Red))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Red))]
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
}

impl ModuleInfoProvider for NetstatConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("netstat"),
            schema: || schema_for!(NetstatConfig),
            layout_id: Some(String::from("netstat")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(NetstatConfig);
