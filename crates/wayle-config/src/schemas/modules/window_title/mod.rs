mod icons;

use std::collections::HashMap;

use schemars::schema_for;
use wayle_derive::wayle_config;

pub use self::icons::BUILTIN_MAPPINGS;
use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Window title module configuration.
#[wayle_config(bar_button)]
pub struct WindowTitleConfig {
    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ title }}` - Window title
    /// - `{{ app }}` - Application name (WM_CLASS on Hyprland)
    ///
    /// ## Examples
    ///
    /// - `"{{ title }}"` - "README.md - VSCode"
    /// - `"{{ app }}: {{ title }}"` - "firefox: GitHub"
    #[i18n("settings-modules-window-title-format")]
    #[default(String::from("{{ title }}"))]
    pub format: ConfigProperty<String>,

    /// Fallback icon when no mapping matches.
    #[serde(rename = "icon-name")]
    #[i18n("settings-modules-window-title-icon-name")]
    #[default(String::from("ld-app-window-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Icon mappings. Glob patterns to icon names.
    ///
    /// Patterns match window class by default. Prefix with `title:` to match
    /// window title instead. User mappings are checked before built-in mappings.
    #[serde(rename = "icon-mappings")]
    #[i18n("settings-modules-window-title-icon-mappings")]
    #[default(HashMap::new())]
    pub icon_mappings: ConfigProperty<HashMap<String, String>>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-window-title-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-window-title-border-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-window-title-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-window-title-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-window-title-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-window-title-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-window-title-label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-window-title-label-max-length")]
    #[default(50)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-window-title-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-window-title-left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-window-title-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-window-title-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-window-title-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-window-title-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for WindowTitleConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("window-title"),
            icon: String::from("󱂬"),
            description: String::from("Active window title display"),
            behavior_configs: vec![(String::from("window-title"), || {
                schema_for!(WindowTitleConfig)
            })],
            styling_configs: vec![],
        }
    }
}
