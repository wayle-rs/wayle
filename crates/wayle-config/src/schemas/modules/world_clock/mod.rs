use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// World clock module configuration.
#[wayle_config(bar_button)]
pub struct WorldClockConfig {
    /// Format string with embedded timezone blocks.
    ///
    /// ## Syntax
    ///
    /// Use `{{ tz('timezone', 'strftime') }}` syntax to insert formatted times.
    /// Text outside placeholders is preserved as literal text.
    ///
    /// ## Examples
    ///
    /// - `"{{ tz('UTC', '%H:%M %Z') }}"` - "14:30 UTC"
    /// - `"NYC {{ tz('America/New_York', '%H:%M') }}  TYO {{ tz('Asia/Tokyo', '%H:%M') }}"` - "NYC 09:30  TYO 23:30"
    /// - `"{{ tz('America/New_York', '%H:%M %Z') }} | {{ tz('Europe/London', '%H:%M %Z') }}"` - "09:30 EST | 14:30 GMT"
    #[i18n("settings-modules-world-clock-format")]
    #[default(String::from("{{ tz('UTC', '%H:%M %Z') }}"))]
    pub format: ConfigProperty<String>,

    /// Symbolic icon name.
    #[serde(rename = "icon-name")]
    #[i18n("settings-modules-world-clock-icon-name")]
    #[default(String::from("ld-globe-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-world-clock-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-world-clock-border-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-world-clock-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-world-clock-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-world-clock-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-world-clock-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-world-clock-label-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-world-clock-label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-world-clock-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-world-clock-left-click")]
    #[default(ClickAction::None)]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-world-clock-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-world-clock-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-world-clock-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-world-clock-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

impl ModuleInfoProvider for WorldClockConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("world-clock"),
            icon: String::from("󱉊"),
            description: String::from("World clock with multiple timezone support"),
            behavior_configs: vec![(String::from("world-clock"), || {
                schema_for!(WorldClockConfig)
            })],
            styling_configs: vec![],
        }
    }
}
