use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// Microphone module configuration.
#[wayle_config(bar_button)]
pub struct MicrophoneConfig {
    /// Icon shown when microphone is active (unmuted).
    #[serde(rename = "icon-active")]
    #[default(String::from("ld-mic-symbolic"))]
    pub icon_active: ConfigProperty<String>,

    /// Icon shown when microphone is muted.
    #[serde(rename = "icon-muted")]
    #[default(String::from("ld-mic-off-symbolic"))]
    pub icon_muted: ConfigProperty<String>,

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

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Red))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display percentage label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Red))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("audio")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click. Default toggles input mute.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::Shell(String::from("wayle audio input-mute")))]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,

    /// Dynamic color thresholds based on microphone volume percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `above` for high-value warnings (e.g., high input gain).
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.microphone.thresholds]]
    /// above = 70
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    ///
    /// [[modules.microphone.thresholds]]
    /// above = 90
    /// icon-color = "status-error"
    /// label-color = "status-error"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for MicrophoneConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("microphone"),
            icon: String::from(""),
            description: String::from("Microphone input control and mute toggle"),
            behavior_configs: vec![(String::from("microphone"), || schema_for!(MicrophoneConfig))],
            styling_configs: vec![],
        }
    }
}
