use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::{
        modules::TimeFormat,
        styling::{ColorValue, CssToken},
    },
};

/// Weather module configuration.
#[wayle_config(bar_button, i18n_prefix = "settings-modules-weather")]
pub struct WeatherConfig {
    /// Weather data provider.
    #[default(WeatherProvider::default())]
    pub provider: ConfigProperty<WeatherProvider>,

    /// Location for weather data (city name or "lat,lon" coordinates).
    #[default(String::from("San Francisco"))]
    pub location: ConfigProperty<String>,

    /// Temperature unit.
    #[default(TemperatureUnit::default())]
    pub units: ConfigProperty<TemperatureUnit>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ temp }}` - Current temperature (e.g., "72")
    /// - `{{ temp_unit }}` - Temperature unit symbol ("°F" or "°C")
    /// - `{{ feels_like }}` - Feels-like temperature
    /// - `{{ condition }}` - Weather condition text (e.g., "Cloudy")
    /// - `{{ humidity }}` - Humidity percentage (e.g., "65%")
    /// - `{{ wind_speed }}` - Wind speed with unit (e.g., "12 km/h")
    /// - `{{ wind_dir }}` - Wind direction (e.g., "NW")
    /// - `{{ high }}` - Today's high temperature
    /// - `{{ low }}` - Today's low temperature
    ///
    /// ## Examples
    ///
    /// - `"{{ temp }}{{ temp_unit }}"` - "22°C"
    /// - `"{{ temp }}{{ temp_unit }} {{ condition }}"` - "22°C Partly Cloudy"
    /// - `"{{ temp }}{{ temp_unit }} H:{{ high }} L:{{ low }}"` - "22°C H:25 L:18"
    #[default(String::from("{{ temp }}{{ temp_unit }}"))]
    pub format: ConfigProperty<String>,

    /// Time display format for sunrise/sunset and hourly forecast.
    #[serde(rename = "time-format")]
    #[default(TimeFormat::default())]
    pub time_format: ConfigProperty<TimeFormat>,

    /// Polling interval in seconds.
    #[serde(rename = "refresh-interval-seconds")]
    #[default(1800)]
    pub refresh_interval_seconds: ConfigProperty<u32>,

    /// Visual Crossing API key. Supports `$VAR_NAME` syntax to reference
    /// environment variables from `.*.env` files in the config directory.
    #[serde(rename = "visual-crossing-key")]
    #[default(None)]
    pub visual_crossing_key: ConfigProperty<Option<String>>,

    /// WeatherAPI.com API key. Supports `$VAR_NAME` syntax to reference
    /// environment variables from `.*.env` files in the config directory.
    #[serde(rename = "weatherapi-key")]
    #[default(None)]
    pub weatherapi_key: ConfigProperty<Option<String>>,

    /// Fallback icon for weather.
    #[serde(rename = "icon-name")]
    #[default(String::from("ld-sun-symbolic"))]
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

    /// Display temperature label.
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
    #[default(ClickAction::Dropdown(String::from("weather")))]
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

/// Weather data provider selection.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    wayle_derive::EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum WeatherProvider {
    /// Open-Meteo (no API key required).
    #[default]
    OpenMeteo,
    /// Visual Crossing (requires API key).
    VisualCrossing,
    /// WeatherAPI.com (requires API key).
    WeatherApi,
}

/// Temperature unit for display.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    wayle_derive::EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum TemperatureUnit {
    /// Celsius (metric).
    #[default]
    Metric,
    /// Fahrenheit (imperial).
    Imperial,
}

impl ModuleInfoProvider for WeatherConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("weather"),
            icon: String::from("󰖐"),
            description: String::from("Weather display with forecasts"),
            behavior_configs: vec![(String::from("weather"), || schema_for!(WeatherConfig))],
            styling_configs: vec![],
        }
    }
}
