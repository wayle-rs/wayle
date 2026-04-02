use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Bluetooth module configuration.
#[wayle_config(bar_button)]
pub struct BluetoothConfig {
    /// Icon when Bluetooth is disabled or unavailable.
    #[serde(rename = "disabled-icon")]
    #[i18n("settings-modules-bluetooth-disabled-icon")]
    #[default(String::from("ld-bluetooth-off-symbolic"))]
    pub disabled_icon: ConfigProperty<String>,

    /// Icon when Bluetooth is on but no devices connected.
    #[serde(rename = "disconnected-icon")]
    #[i18n("settings-modules-bluetooth-disconnected-icon")]
    #[default(String::from("ld-bluetooth-symbolic"))]
    pub disconnected_icon: ConfigProperty<String>,

    /// Icon when devices are connected.
    #[serde(rename = "connected-icon")]
    #[i18n("settings-modules-bluetooth-connected-icon")]
    #[default(String::from("ld-bluetooth-connected-symbolic"))]
    pub connected_icon: ConfigProperty<String>,

    /// Icon when scanning for devices.
    #[serde(rename = "searching-icon")]
    #[i18n("settings-modules-bluetooth-searching-icon")]
    #[default(String::from("ld-bluetooth-searching-symbolic"))]
    pub searching_icon: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-bluetooth-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-bluetooth-border-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-bluetooth-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-bluetooth-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-bluetooth-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display connection label (device name or count).
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-bluetooth-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-bluetooth-label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-bluetooth-label-max-length")]
    #[default(15)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-bluetooth-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-bluetooth-left-click")]
    #[default(ClickAction::Dropdown(String::from("bluetooth")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-bluetooth-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-bluetooth-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-bluetooth-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-bluetooth-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for BluetoothConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("bluetooth"),
            icon: String::from("󰂯"),
            description: String::from("Bluetooth connection status"),
            behavior_configs: vec![(String::from("bluetooth"), || schema_for!(BluetoothConfig))],
            styling_configs: vec![],
        }
    }
}
