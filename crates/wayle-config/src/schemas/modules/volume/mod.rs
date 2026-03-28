use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Icon source for app volume entries in the dropdown.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum AppIconSource {
    /// Wayle's curated symbolic icons matched by app name.
    #[default]
    Mapped,
    /// Native application icons reported by PulseAudio.
    Native,
}

/// Volume module configuration.
#[wayle_config(bar_button)]
pub struct VolumeConfig {
    /// Icons for volume levels from low to maximum.
    ///
    /// The percentage is divided evenly among icons. With 3 icons:
    /// 1-33% uses icons\[0\], 34-66% uses icons\[1\], 67-100% uses icons\[2\].
    #[serde(rename = "level-icons")]
    #[default(vec![
        String::from("ld-volume-symbolic"),
        String::from("ld-volume-1-symbolic"),
        String::from("ld-volume-2-symbolic"),
    ])]
    pub level_icons: ConfigProperty<Vec<String>>,

    /// Icon shown when audio output is muted.
    #[serde(rename = "icon-muted")]
    #[default(String::from("ld-volume-x-symbolic"))]
    pub icon_muted: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::Red))]
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
    #[default(ColorValue::Token(CssToken::Red))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display percentage label.
    #[serde(rename = "label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[default(ColorValue::Token(CssToken::Red))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ percent }}` - Volume (0-100)
    ///
    /// ## Examples
    ///
    /// - `"{{ percent }}%"` - "45%"
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

    /// Action on left click. Default opens the audio dropdown.
    #[serde(rename = "left-click")]
    #[default(ClickAction::Dropdown(String::from("audio")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click. Default toggles mute.
    #[serde(rename = "middle-click")]
    #[default(ClickAction::Shell(String::from("wayle audio output-mute")))]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,

    /// Icon source for app volume entries in the audio dropdown.
    #[serde(rename = "dropdown-app-icons")]
    #[default(AppIconSource::Mapped)]
    pub dropdown_app_icons: ConfigProperty<AppIconSource>,
}

impl ModuleInfoProvider for VolumeConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("volume"),
            icon: String::from("󰕾"),
            description: String::from("Audio volume control and mute toggle"),
            behavior_configs: vec![(String::from("volume"), || schema_for!(VolumeConfig))],
            styling_configs: vec![],
        }
    }
}
