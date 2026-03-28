use std::collections::HashMap;

use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Network module configuration.
#[wayle_config(bar_button)]
pub struct NetworkConfig {
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

    /// WiFi signal strength icons from weak to excellent.
    ///
    /// The signal percentage maps to icons: 0-25% uses icons\[0\], 26-50% uses
    /// icons\[1\], etc.
    #[serde(rename = "wifi-signal-icons")]
    #[default(vec![
        String::from("cm-wireless-signal-weak-symbolic"),
        String::from("cm-wireless-signal-ok-symbolic"),
        String::from("cm-wireless-signal-good-symbolic"),
        String::from("cm-wireless-signal-excellent-symbolic"),
    ])]
    pub wifi_signal_icons: ConfigProperty<Vec<String>>,

    /// Wired icon when connected.
    #[serde(rename = "wired-connected-icon")]
    #[default(String::from("cm-wired-symbolic"))]
    pub wired_connected_icon: ConfigProperty<String>,

    /// Wired icon when connecting.
    #[serde(rename = "wired-acquiring-icon")]
    #[default(String::from("cm-wired-acquiring-symbolic"))]
    pub wired_acquiring_icon: ConfigProperty<String>,

    /// Wired icon when disconnected.
    #[serde(rename = "wired-disconnected-icon")]
    #[default(String::from("cm-wired-disconnected-symbolic"))]
    pub wired_disconnected_icon: ConfigProperty<String>,

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

    /// Display connection label (SSID for WiFi, "Wired" for ethernet).
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
    #[default(ClickAction::Dropdown(String::from("network")))]
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

    /// Custom display names for VPN tunnels, keyed by connection UUID.
    ///
    /// ## Examples
    ///
    /// ```toml
    /// [modules.network.vpn-aliases]
    /// "a1b2c3d4-e5f6-7890-abcd-ef1234567890" = "Work VPN"
    /// "f0e1d2c3-b4a5-6789-0abc-def123456789" = "Home Server"
    /// ```
    #[serde(rename = "vpn-aliases")]
    #[default(HashMap::new())]
    pub vpn_aliases: ConfigProperty<HashMap<String, String>>,
}

impl ModuleInfoProvider for NetworkConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("network"),
            icon: String::from("󰖩"),
            description: String::from("Network connection status"),
            behavior_configs: vec![(String::from("network"), || schema_for!(NetworkConfig))],
            styling_configs: vec![],
        }
    }
}
