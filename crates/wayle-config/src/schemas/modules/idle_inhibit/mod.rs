use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Idle inhibitor module configuration.
///
/// Prevents screen dimming, lock, and suspend when active.
/// Can control via CLI: `wayle idle on/off/duration/remaining/status`
#[wayle_config(bar_button)]
pub struct IdleInhibitConfig {
    /// Duration in minutes when service starts. 0 means indefinite.
    #[serde(rename = "startup-duration")]
    #[i18n("settings-modules-idle-inhibit-startup-duration")]
    #[default(60)]
    pub startup_duration: ConfigProperty<u32>,

    /// Icon when idle inhibitor is inactive.
    #[serde(rename = "icon-inactive")]
    #[i18n("settings-modules-idle-inhibit-icon-inactive")]
    #[default(String::from("tb-coffee-off-symbolic"))]
    pub icon_inactive: ConfigProperty<String>,

    /// Icon when idle inhibitor is active.
    #[serde(rename = "icon-active")]
    #[i18n("settings-modules-idle-inhibit-icon-active")]
    #[default(String::from("tb-coffee-symbolic"))]
    pub icon_active: ConfigProperty<String>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ state }}` - Inhibitor state text (On, Off)
    /// - `{{ remaining }}` - Time remaining (e.g., "45m", shows "--" when indefinite)
    /// - `{{ duration }}` - Total duration (e.g., "60m", shows "--" when indefinite)
    ///
    /// ## Examples
    ///
    /// - `"{{ state }}"` - "On"
    /// - `"{{ remaining }}/{{ duration }}"` - "45m/60m"
    /// - `"{{ state }} ({{ remaining }})"` - "On (45m)"
    #[serde(rename = "format")]
    #[i18n("settings-modules-idle-inhibit-format")]
    #[default(String::from("{{ state }}"))]
    pub format: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-idle-inhibit-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-idle-inhibit-border-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-idle-inhibit-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-idle-inhibit-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-idle-inhibit-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-idle-inhibit-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-idle-inhibit-label-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-idle-inhibit-label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-idle-inhibit-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click. Default toggles indefinite idle inhibit.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-idle-inhibit-left-click")]
    #[default(ClickAction::Shell(String::from("wayle idle toggle --indefinite")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click. Default toggles timed idle inhibit.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-idle-inhibit-right-click")]
    #[default(ClickAction::Shell(String::from("wayle idle toggle")))]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-idle-inhibit-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-idle-inhibit-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-idle-inhibit-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for IdleInhibitConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("idle-inhibit"),
            icon: String::from(""),
            description: String::from("Prevent screen dimming, lock, and suspend"),
            behavior_configs: vec![(String::from("idle-inhibit"), || {
                schema_for!(IdleInhibitConfig)
            })],
            styling_configs: vec![],
        }
    }
}
