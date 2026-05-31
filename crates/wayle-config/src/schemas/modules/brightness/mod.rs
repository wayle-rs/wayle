use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ThresholdEntry},
};

/// Backlight control bar module.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-brightness")]
pub struct BrightnessConfig {
    /// Icons for brightness levels from low to maximum.
    ///
    /// The percentage is divided evenly among icons. With 3 icons:
    /// 0-33% uses icons\[0\], 34-66% uses icons\[1\], 67-100% uses icons\[2\].
    #[serde(rename = "level-icons")]
    #[default(vec![
        String::from("ld-sun-dim-symbolic"),
        String::from("ld-sun-medium-symbolic"),
        String::from("ld-sun-symbolic"),
    ])]
    pub level_icons: ConfigProperty<Vec<String>>,

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

    /// Display percentage label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - Brightness (0-100)
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "65%"
    #[serde(rename = "format")]
    #[default(String::from("{{ percent }}%"))]
    pub format: ConfigProperty<String>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click. Default opens the brightness dropdown.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("brightness")))]
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

    /// Dynamic color thresholds based on brightness percentage.
    ///
    /// Entries are checked in order; the last matching entry wins for each
    /// color slot. Use `below` for low-brightness warnings.
    ///
    /// ## Example
    ///
    /// ```toml
    /// [[modules.brightness.thresholds]]
    /// below = 20
    /// icon-color = "status-warning"
    /// label-color = "status-warning"
    /// ```
    #[serde(rename = "thresholds")]
    #[default(Vec::new())]
    pub thresholds: ConfigProperty<Vec<ThresholdEntry>>,
}

impl ModuleInfoProvider for BrightnessConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("brightness"),
            schema: || schema_for!(BrightnessConfig),
            layout_id: Some(String::from("brightness")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(BrightnessConfig);
