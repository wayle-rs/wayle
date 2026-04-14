use std::collections::HashMap;

use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Keyboard input module configuration.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-keyboard-input")]
pub struct KeyboardInputConfig {
    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ layout }}` - Raw layout name from the compositor (e.g., "English (US)")
    /// - `{{ alias }}` - User-defined alias from `layout-alias-map`, falls back to `{{ layout }}`
    ///
    /// ## Examples
    ///
    /// - `"{{ layout }}"` - "English (US)"
    /// - `"{{ alias }}"` - "EN" (with alias map configured)
    #[default(String::from("{{ alias }}"))]
    pub format: ConfigProperty<String>,

    /// Symbolic icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-keyboard-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
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
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
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
    #[default(ClickAction::None)]
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

    /// Language name mapping.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [modules.keyboard-input.layout-alias-map]
    /// "English (US)" = "EN"
    /// "Czech (QWERTY)" = "Czech"
    /// ```
    #[serde(rename = "layout-alias-map")]
    #[default(HashMap::new())]
    pub layout_alias_map: ConfigProperty<HashMap<String, String>>,
}

impl ModuleInfoProvider for KeyboardInputConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("keyboard-input"),
            icon: String::from("󰌌"),
            description: String::from("Keyboard layout indicator"),
            behavior_configs: vec![(String::from("keyboard-input"), || {
                schema_for!(KeyboardInputConfig)
            })],
            styling_configs: vec![],
        }
    }
}
