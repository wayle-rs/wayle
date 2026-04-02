mod icons;

use std::collections::HashMap;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

pub use self::icons::BUILTIN_MAPPINGS;
use crate::{
    ClickAction, ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken},
};

/// Media player module configuration.
///
#[wayle_config(bar_button)]
pub struct MediaConfig {
    /// Icon display mode.
    #[serde(rename = "icon-type")]
    #[i18n("settings-modules-media-icon-type")]
    #[default(MediaIconType::ApplicationMapped)]
    pub icon_type: ConfigProperty<MediaIconType>,

    /// Custom player-to-icon mappings for application-mapped mode.
    ///
    /// Keys are glob patterns matching MPRIS bus names, values are icon names.
    /// These override built-in mappings when matched.
    #[serde(rename = "player-icons")]
    #[i18n("settings-modules-media-player-icons")]
    #[default(HashMap::new())]
    pub player_icons: ConfigProperty<HashMap<String, String>>,

    /// Player bus name patterns to exclude from discovery.
    /// This property requires a restart to take effect.
    #[serde(rename = "players-ignored")]
    #[i18n("settings-modules-media-players-ignored")]
    #[default(Vec::new())]
    pub players_ignored: ConfigProperty<Vec<String>>,

    /// Preferred player priority order as glob patterns matching bus names.
    ///
    /// When no player is manually selected, this determines which player
    /// becomes active. Patterns are checked in order; first match wins.
    /// If no pattern matches, the first playing player is selected.
    #[serde(rename = "player-priority")]
    #[i18n("settings-modules-media-player-priority")]
    #[default(Vec::new())]
    pub player_priority: ConfigProperty<Vec<String>>,

    /// Format string for the label.
    ///
    /// ## Placeholders
    ///
    /// - `{{ title }}` - Track title
    /// - `{{ artist }}` - Artist name(s)
    /// - `{{ album }}` - Album name
    /// - `{{ status }}` - Playback status text (Playing, Paused, Stopped)
    /// - `{{ status_icon }}` - Playback status icon character
    ///
    /// ## Examples
    ///
    /// - `"{{ title }} - {{ artist }}"` - "Bohemian Rhapsody - Queen"
    /// - `"{{ status_icon }} {{ title }}"` - "▶ Bohemian Rhapsody"
    /// - `"{{ artist }}: {{ title }} ({{ album }})"` - "Queen: Bohemian Rhapsody (A Night at the Opera)"
    #[i18n("settings-modules-media-format")]
    #[default(String::from("{{ title }} - {{ artist }}"))]
    pub format: ConfigProperty<String>,

    /// Symbolic icon name for default mode.
    #[serde(rename = "icon-name")]
    #[i18n("settings-modules-media-icon-name")]
    #[default(String::from("ld-music-symbolic"))]
    pub icon_name: ConfigProperty<String>,

    /// Icon shown for spinning-disc mode.
    #[serde(rename = "spinning-disc-icon")]
    #[i18n("settings-modules-media-spinning-disc-icon")]
    #[default(String::from("ld-disc-3-symbolic"))]
    pub spinning_disc_icon: ConfigProperty<String>,

    /// Display border around button.
    #[serde(rename = "border-show")]
    #[i18n("settings-modules-media-border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[i18n("settings-modules-media-border-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Display module icon.
    #[serde(rename = "icon-show")]
    #[i18n("settings-modules-media-icon-show")]
    #[default(true)]
    pub icon_show: ConfigProperty<bool>,

    /// Icon foreground color. Auto selects based on variant for contrast.
    #[serde(rename = "icon-color")]
    #[i18n("settings-modules-media-icon-color")]
    #[default(ColorValue::Auto)]
    pub icon_color: ConfigProperty<ColorValue>,

    /// Icon container background color token.
    #[serde(rename = "icon-bg-color")]
    #[i18n("settings-modules-media-icon-bg-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub icon_bg_color: ConfigProperty<ColorValue>,

    /// Display text label.
    #[serde(rename = "label-show")]
    #[i18n("settings-modules-media-label-show")]
    #[default(true)]
    pub label_show: ConfigProperty<bool>,

    /// Label text color token.
    #[serde(rename = "label-color")]
    #[i18n("settings-modules-media-label-color")]
    #[default(ColorValue::Token(CssToken::Blue))]
    pub label_color: ConfigProperty<ColorValue>,

    /// Max label characters before truncation with ellipsis. Set to 0 to disable.
    #[serde(rename = "label-max-length")]
    #[i18n("settings-modules-media-label-max-length")]
    #[default(35)]
    pub label_max_length: ConfigProperty<u32>,

    /// Button background color token.
    #[serde(rename = "button-bg-color")]
    #[i18n("settings-modules-media-button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,

    /// Action on left click.
    #[serde(rename = "left-click")]
    #[i18n("settings-modules-media-left-click")]
    #[default(ClickAction::Dropdown(String::from("media")))]
    pub left_click: ConfigProperty<ClickAction>,

    /// Action on right click.
    #[serde(rename = "right-click")]
    #[i18n("settings-modules-media-right-click")]
    #[default(ClickAction::None)]
    pub right_click: ConfigProperty<ClickAction>,

    /// Action on middle click.
    #[serde(rename = "middle-click")]
    #[i18n("settings-modules-media-middle-click")]
    #[default(ClickAction::None)]
    pub middle_click: ConfigProperty<ClickAction>,

    /// Action on scroll up.
    #[serde(rename = "scroll-up")]
    #[i18n("settings-modules-media-scroll-up")]
    #[default(ClickAction::None)]
    pub scroll_up: ConfigProperty<ClickAction>,

    /// Action on scroll down.
    #[serde(rename = "scroll-down")]
    #[i18n("settings-modules-media-scroll-down")]
    #[default(ClickAction::None)]
    pub scroll_down: ConfigProperty<ClickAction>,
}

/// Icon display mode for the media module.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum MediaIconType {
    /// Static icon from icon-name field.
    Default,
    /// Dynamic icon from media player's desktop entry, falling back to icon-name.
    Application,
    /// Spinning disc icon that animates during playback. Uses slightly more CPU.
    SpinningDisc,
    /// Maps player to icon via glob patterns, with built-in mappings for common players.
    #[default]
    ApplicationMapped,
}

impl MediaIconType {
    /// Returns the kebab-case string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Application => "application",
            Self::SpinningDisc => "spinning-disc",
            Self::ApplicationMapped => "application-mapped",
        }
    }
}

impl ModuleInfoProvider for MediaConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("media"),
            icon: String::from("󰎆"),
            description: String::from("Media player controls and now playing info"),
            behavior_configs: vec![(String::from("media"), || schema_for!(MediaConfig))],
            styling_configs: vec![],
        }
    }
}
