use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Mullvad VPN connection status with a dropdown for choosing a relay.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-mullvad")]
pub struct MullvadConfig {
    /// Icon when connected to a relay.
    #[serde(rename = "connected-icon")]
    #[default(String::from("network-vpn-symbolic"))]
    pub connected_icon: ConfigProperty<String>,

    /// Icon while a tunnel is being established.
    #[serde(rename = "connecting-icon")]
    #[default(String::from("network-vpn-acquiring-symbolic"))]
    pub connecting_icon: ConfigProperty<String>,

    /// Icon when disconnected.
    #[serde(rename = "disconnected-icon")]
    #[default(String::from("network-vpn-disconnected-symbolic"))]
    pub disconnected_icon: ConfigProperty<String>,

    /// Icon when the daemon is in a blocked or error state.
    #[serde(rename = "blocked-icon")]
    #[default(String::from("network-vpn-no-route-symbolic"))]
    pub blocked_icon: ConfigProperty<String>,

    /// Icon when logged out or the daemon is unavailable.
    #[serde(rename = "disabled-icon")]
    #[default(String::from("network-vpn-disabled-symbolic"))]
    pub disabled_icon: ConfigProperty<String>,

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

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display connection label (connected city or status).
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Green))]
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
    #[default(ClickAction::Dropdown(String::from("mullvad")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click. Defaults to `:toggle`, which connects when off
    /// and disconnects otherwise. Set a dropdown/command to override, or `""`
    /// (None) for no action.
    #[serde(rename = "right-click")]
    #[default(ClickAction::Shell(String::from(":toggle")))]
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

impl ModuleInfoProvider for MullvadConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("mullvad"),
            schema: || schema_for!(MullvadConfig),
            layout_id: Some(String::from("mullvad")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(MullvadConfig);
