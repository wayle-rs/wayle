use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Time display with a calendar dropdown.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-clock")]
pub struct ClockConfig {
    /// Format string using strftime syntax.
    ///
    /// ## Common Specifiers
    ///
    /// - `%H` - Hour (00-23)
    /// - `%I` - Hour (01-12)
    /// - `%M` - Minute (00-59)
    /// - `%S` - Second (00-59)
    /// - `%p` - AM/PM
    /// - `%a` - Abbreviated weekday (Mon, Tue)
    /// - `%A` - Full weekday (Monday)
    /// - `%b` - Abbreviated month (Jan, Feb)
    /// - `%B` - Full month (January)
    /// - `%d` - Day of month (01-31)
    /// - `%Y` - Year (2024)
    ///
    /// ## Examples
    ///
    /// - `"%H:%M"` - "14:30"
    /// - `"%I:%M %p"` - "02:30 PM"
    /// - `"%a %b %d %I:%M %p"` - "Mon Jan 15 02:30 PM"
    #[default(String::from("%a %b %d %I:%M %p"))]
    pub format: ConfigProperty<String>,

    /// Symbolic icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("tb-calendar-time-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
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

    /// Display text label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
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
    #[default(ClickAction::Dropdown(String::from("calendar")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(ClickAction::Dropdown(String::from("weather")))]
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

    /// Show seconds in the calendar dropdown clock display.
    #[serde(rename = "dropdown-show-seconds")]
    #[default(false)]
    pub dropdown_show_seconds: ConfigProperty<bool>,
}

impl ModuleInfoProvider for ClockConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("clock"),
            schema: || schema_for!(ClockConfig),
            layout_id: Some(String::from("clock")),
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::bar_button()
    }
}

crate::register_module!(ClockConfig);
