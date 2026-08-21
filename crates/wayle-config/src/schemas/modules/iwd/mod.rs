use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// WiFi connection status (via IWD) with a dropdown for switching networks.
///
/// Use this instead of the `network` module on systems where WiFi is managed by
/// `iwd` rather than NetworkManager. WiFi only — IWD does not manage Ethernet.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-iwd")]
pub struct IwdConfig {
    /// WiFi icon when disabled.
    #[serde(rename = "wifi-disabled-icon")]
    #[default(String::from("cm-wireless-disabled-symbolic"))]
    pub wifi_disabled_icon: ConfigProperty<String>,

    /// WiFi icon when connecting.
    #[serde(rename = "wifi-acquiring-icon")]
    #[default(String::from("cm-wireless-acquiring-symbolic"))]
    pub wifi_acquiring_icon: ConfigProperty<String>,

    /// WiFi icon when disconnected.
    #[serde(rename = "wifi-offline-icon")]
    #[default(String::from("cm-wireless-offline-symbolic"))]
    pub wifi_offline_icon: ConfigProperty<String>,

    /// WiFi icon when connected but signal strength unavailable.
    #[serde(rename = "wifi-connected-icon")]
    #[default(String::from("cm-wireless-connected-symbolic"))]
    pub wifi_connected_icon: ConfigProperty<String>,

    /// WiFi signal strength icons from weakest to strongest.
    ///
    /// The signal-strength bucket is scaled across the list: `icons[0]` is used
    /// for the weakest signal and the last entry for the strongest.
    #[serde(rename = "wifi-signal-icons")]
    #[default(vec![
        String::from("cm-wireless-signal-none-symbolic"),
        String::from("cm-wireless-signal-weak-symbolic"),
        String::from("cm-wireless-signal-ok-symbolic"),
        String::from("cm-wireless-signal-good-symbolic"),
        String::from("cm-wireless-signal-excellent-symbolic"),
    ])]
    pub wifi_signal_icons: ConfigProperty<Vec<String>>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
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
    #[default(ColorValue::Token(CssToken::Accent))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display connection label (SSID for WiFi).
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(15)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("iwd")))]
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

impl ModuleInfoProvider for IwdConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("iwd"),
            schema: || schema_for!(IwdConfig),
            layout_id: Some(String::from("iwd")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(IwdConfig);
