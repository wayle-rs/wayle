use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Keybind mode indicator configuration.
#[wayle_config(bar_button)]
pub struct KeybindModeConfig {
    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ mode }}` - Current keybind mode name (shows "default" when inactive)
    ///
    /// ## Examples
    ///
    /// - `"{{ mode }}"` - "resize"
    /// - `"Mode: {{ mode }}"` - "Mode: resize"
    /// - `"[{{ mode }}]"` - "[resize]"
    #[i18n("settings-modules-keybind-mode-format")]
    #[default(String::from("{{ mode }}"))]
    pub format: ConfigProperty<String>,

    /// Symbolic icon name.
    #[serde(rename = "icon-name")]
    #[i18n("settings-modules-keybind-mode-icon-name")]
    #[default(String::from("ld-layers-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Automatically hide module when no mode is active.
    #[serde(rename = "auto-hide")]
    #[i18n("settings-modules-keybind-mode-auto-hide")]
    #[default(false)]
    pub auto_hide: ConfigProperty<bool>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-keybind-mode-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-keybind-mode-border-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-keybind-mode-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-keybind-mode-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-keybind-mode-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-keybind-mode-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-keybind-mode-label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-keybind-mode-label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-keybind-mode-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-keybind-mode-left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-keybind-mode-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-keybind-mode-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-keybind-mode-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-keybind-mode-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for KeybindModeConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("keybind-mode"),
            icon: String::from("󰌨"),
            description: String::from("Keybind mode indicator"),
            behavior_configs: vec![(String::from("keybind-mode"), || {
                schema_for!(KeybindModeConfig)
            })],
            styling_configs: vec![],
        }
    }
}
