use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Dashboard module configuration.
#[wayle_config]
pub struct DashboardConfig {
    /// Override the auto-detected distro icon.
    #[serde(rename = "icon-override")]
    #[i18n("settings-modules-dashboard-icon-override")]
    #[default(String::new())]
    pub icon_override: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-dashboard-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-dashboard-border-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-dashboard-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-dashboard-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-dashboard-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-dashboard-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-dashboard-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-dashboard-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-dashboard-left-click")]
    #[default(ClickAction::Dropdown(String::from("dashboard")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Shell command for the lock button in the dashboard dropdown.
    #[serde(rename = "dropdown-lock-command")]
    #[i18n("settings-modules-dashboard-dropdown-lock-command")]
    #[default(String::from("loginctl lock-session"))]
    pub dropdown_lock_command: ConfigProperty<String>,

    /// Shell command for the logout button in the dashboard dropdown.
    #[serde(rename = "dropdown-logout-command")]
    #[i18n("settings-modules-dashboard-dropdown-logout-command")]
    #[default(String::from("loginctl terminate-session $XDG_SESSION_ID"))]
    pub dropdown_logout_command: ConfigProperty<String>,

    /// Shell command for the reboot button in the dashboard dropdown.
    #[serde(rename = "dropdown-reboot-command")]
    #[i18n("settings-modules-dashboard-dropdown-reboot-command")]
    #[default(String::from("systemctl reboot"))]
    pub dropdown_reboot_command: ConfigProperty<String>,

    /// Shell command for the power-off button in the dashboard dropdown.
    #[serde(rename = "dropdown-poweroff-command")]
    #[i18n("settings-modules-dashboard-dropdown-poweroff-command")]
    #[default(String::from("systemctl poweroff"))]
    pub dropdown_poweroff_command: ConfigProperty<String>,

    /// Hidden: icon always shown.
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[i18n(skip)]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Hidden: label visibility (always false).
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[i18n(skip)]
    #[default(false)]
    pub label_show: ConfigProperty<bool>,

    /// Hidden: label color (unused).
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[i18n(skip)]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Hidden: label max length (unused).
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[i18n(skip)]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Hidden: button background (unused).
    #[serde(skip)]
    #[schemars(skip)]
    #[wayle(skip)]
    #[i18n(skip)]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,
}

impl ModuleInfoProvider for DashboardConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("dashboard"),
            icon: String::from("󰕰"),
            description: String::from("Quick access dashboard with distro icon"),
            behavior_configs: vec![(String::from("dashboard"), || schema_for!(DashboardConfig))],
            styling_configs: vec![],
        }
    }
}
