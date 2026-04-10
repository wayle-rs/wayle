use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Pomodoro timer module configuration.
#[wayle_config(bar_button)]
pub struct PomodoroConfig {
    /// Work session duration in minutes.
    #[serde(rename = "work-duration")]
    #[default(25)]
    pub work_duration: ConfigProperty<u32>,

    /// Short break duration in minutes.
    #[serde(rename = "short-break-duration")]
    #[default(5)]
    pub short_break_duration: ConfigProperty<u32>,

    /// Long break duration in minutes.
    #[serde(rename = "long-break-duration")]
    #[default(15)]
    pub long_break_duration: ConfigProperty<u32>,

    /// Number of work cycles before a long break.
    #[serde(rename = "cycles-before-long-break")]
    #[default(4)]
    pub cycles_before_long_break: ConfigProperty<u32>,

    /// Symbolic icon name.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-clock-symbolic"))]
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
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Accent color for work sessions in the bar module.
    #[serde(rename = "work-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub work_color: ConfigProperty<ColorValue>,

    /// Accent color for short breaks in the bar module.
    #[serde(rename = "short-break-color")]
    #[default(ColorValue::Token(CssToken::Green))]
    pub short_break_color: ConfigProperty<ColorValue>,

    /// Accent color for long breaks in the bar module.
    #[serde(rename = "long-break-color")]
    #[default(ColorValue::Token(CssToken::Yellow))]
    pub long_break_color: ConfigProperty<ColorValue>,

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
    #[default(ClickAction::Dropdown(String::from("pomodoro")))]
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
}

impl ModuleInfoProvider for PomodoroConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("pomodoro"),
            icon: String::from("󱫢"),
            description: String::from("Pomodoro timer for time management"),
            behavior_configs: vec![(String::from("pomodoro"), || schema_for!(PomodoroConfig))],
            styling_configs: vec![],
        }
    }
}
