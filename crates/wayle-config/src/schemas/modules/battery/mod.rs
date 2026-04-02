use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// Battery module configuration.
#[wayle_config(bar_button)]
pub struct BatteryConfig {
    /// Icons for battery levels from empty to full.
    ///
    /// The percentage is divided evenly among icons. With 5 icons:
    /// 0-20% uses icons\[0\], 21-40% uses icons\[1\], etc.
    #[serde(rename = "level-icons")]
    #[i18n("settings-modules-battery-level-icons")]
    #[default(vec![
        String::from("md-battery_android_0-symbolic"),
        String::from("md-battery_android_frame_1-symbolic"),
        String::from("md-battery_android_frame_2-symbolic"),
        String::from("md-battery_android_frame_3-symbolic"),
        String::from("md-battery_android_frame_4-symbolic"),
        String::from("md-battery_android_frame_5-symbolic"),
        String::from("md-battery_android_frame_6-symbolic"),
        String::from("md-battery_android_frame_full-symbolic"),
    ])]
    pub level_icons: ConfigProperty<Vec<String>>,

    /// Icon shown when battery is charging.
    #[serde(rename = "charging-icon")]
    #[i18n("settings-modules-battery-charging-icon")]
    #[default(String::from("md-battery_android_frame_bolt-symbolic"))]
    pub charging_icon: ConfigProperty<String>,

    /// Icon shown when battery is not present or in an error state.
    #[serde(rename = "alert-icon")]
    #[i18n("settings-modules-battery-alert-icon")]
    #[default(String::from("md-battery_android_alert-symbolic"))]
    pub alert_icon: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-battery-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-battery-border-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-battery-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-battery-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-battery-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display percentage label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-battery-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-battery-label-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - Battery level (0-100)
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
    #[serde(rename = "format")]
    #[i18n("settings-modules-battery-format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-battery-label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-battery-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-battery-left-click")]
    #[default(ClickAction::Dropdown(String::from("battery")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-battery-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-battery-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-battery-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-battery-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,

    /// Dynamic color thresholds based on battery percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `below` for low-value warnings (e.g., low battery).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.battery.thresholds]]
    /// below = 40
    /// icon-color = "status-warning"
    ///
    /// [[modules.battery.thresholds]]
    /// below = 20
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[i18n("settings-modules-battery-thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for BatteryConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("battery"),
            icon: String::from("󰁹"),
            description: String::from("Battery status and charging indicator"),
            behavior_configs: vec![(String::from("battery"), || schema_for!(BatteryConfig))],
            styling_configs: vec![],
        }
    }
}
