use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// System package updates indicator.
///
/// Checks for available updates from official repositories (pacman/checkupdates)
/// and AUR (paru/yay). Displays counts in the bar and provides a dropdown
/// with refresh and update actions.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-updates")]
pub struct UpdatesConfig {
    /// Polling interval in milliseconds.
    #[serde(rename = "poll-interval-ms")]
    #[default(3600000)]
    pub poll_interval_ms: ConfigProperty<u64>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ pacman }}` - Official repository updates count
    /// - `{{ aur }}` - AUR updates count
    /// - `{{ flatpak }}` - Flatpak updates count
    /// - `{{ total }}` - Total updates (pacman + aur + flatpak)
    ///
    /// ## Examples
    ///
    /// - `"{{ total }}"` - "169"
    /// - `"pac:{{ pacman }} aur:{{ aur }}"` - "pac:145 aur:24"
    #[serde(rename = "format")]
    #[default(String::from("{{ total }}"))]
    pub format: ConfigProperty<String>,

    /// Icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("md-package_2-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Hide module when no updates are available.
    #[serde(rename = "hide-if-zero")]
    #[default(false)]
    pub hide_if_zero: ConfigProperty<bool>,

    /// Command to check official repository updates.
    #[serde(rename = "check-official-command")]
    #[default(String::from("checkupdates 2>/dev/null | wc -l"))]
    pub check_official_command: ConfigProperty<String>,

    /// Command to check AUR updates.
    #[serde(rename = "check-aur-command")]
    #[default(String::from("paru -Qua 2>/dev/null | wc -l"))]
    pub check_aur_command: ConfigProperty<String>,

    /// Command to check Flatpak updates.
    #[serde(rename = "check-flatpak-command")]
    #[default(String::from("flatpak remote-ls --updates 2>/dev/null | wc -l"))]
    pub check_flatpak_command: ConfigProperty<String>,

    /// Command to run system update (launched in terminal).
    #[serde(rename = "update-command")]
    #[default(String::from("paru -Syu && flatpak update -y"))]
    pub update_command: ConfigProperty<String>,

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
    #[default(ClickAction::Dropdown(String::from("updates")))]
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

    /// Dynamic color thresholds based on total update count.
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for UpdatesConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("updates"),
            schema: || schema_for!(UpdatesConfig),
            layout_id: Some(String::from("updates")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(UpdatesConfig);
