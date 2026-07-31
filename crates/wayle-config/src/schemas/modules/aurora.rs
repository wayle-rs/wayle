use wayle_derive::wayle_config;
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::{
    ClickAction, ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
    schemas::{
        modules::TimeFormat,
        styling::{ColorValue, CssToken},
    },
};

/// Aurora Borealis forecast module configuration.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-aurora")]
pub struct AuroraConfig {
    #[default(String::from("San Francisco"))]
    pub location: ConfigProperty<String>,

    #[default(String::from("{{ temp }}"))]
    pub format: ConfigProperty<String>,

    #[serde(rename = "time-format")]
    #[default(TimeFormat::default())]
    pub time_format: ConfigProperty<TimeFormat>,

    #[serde(rename = "refresh-interval-seconds")]
    #[default(1800)]
    pub refresh_interval_seconds: ConfigProperty<u32>,

    #[serde(rename = "icon-name")]
    #[default(String::from("ld-aurora-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    #[serde(rename = "icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    #[serde(rename = "icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    #[serde(rename = "icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub label_color: ConfigProperty<ColorValue>,

    #[serde(rename = "label-max-length")]
    #[default(0)]
    pub label_max_length: ConfigProperty<u32>,

    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("aurora")))]
    pub left_click: ConfigProperty<ClickAction>,

    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    #[serde(rename = "middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}
