use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, ScaleFactor, Spacing},
};

/// Systray module configuration.
#[wayle_config(bar_container, i18n_prefix = "settings-modules-systray")]
pub struct SystrayConfig {
    /// Scale multiplier for tray item icons.
    #[serde(rename = "icon-scale")]
    #[default(ScaleFactor::new(1.0))]
    pub icon_scale: ConfigProperty<ScaleFactor>,

    /// Gap between tray items.
    #[serde(rename = "item-gap")]
    #[default(Spacing::new(0.25))]
    pub item_gap: ConfigProperty<Spacing>,

    /// Padding at the ends of the container.
    ///
    /// Applies to left/right edges for horizontal bars, or top/bottom edges
    /// for vertical bars.
    #[serde(rename = "internal-padding")]
    #[default(Spacing::new(0.5))]
    pub internal_padding: ConfigProperty<Spacing>,

    /// Glob patterns for tray items to hide.
    ///
    /// Matches against item ID or title.
    /// Example: `["*discord*", "Steam"]`
    #[default(Vec::new())]
    pub blacklist: ConfigProperty<Vec<String>>,

    /// Custom icon and color overrides.
    ///
    /// First matching override wins. Supports glob patterns.
    ///
    /// ```toml
    /// [[module.systray.overrides]]
    /// name = "*discord*"
    /// icon = "si-discord-symbolic"
    /// color = "blue"
    /// ```
    #[default(Vec::new())]
    pub overrides: ConfigProperty<Vec<TrayItemOverride>>,

    /// Display border around container.
    #[serde(rename = "border-show")]
    #[default(false)]
    pub border_show: ConfigProperty<bool>,

    /// Border color token.
    #[serde(rename = "border-color")]
    #[default(ColorValue::Token(CssToken::BorderAccent))]
    pub border_color: ConfigProperty<ColorValue>,

    /// Container background color token.
    #[serde(rename = "button-bg-color")]
    #[default(ColorValue::Token(CssToken::BgSurfaceElevated))]
    pub button_bg_color: ConfigProperty<ColorValue>,
}

impl ModuleInfoProvider for SystrayConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("systray"),
            icon: String::from("󰆍"),
            description: String::from("System tray icons (StatusNotifierItem)"),
            behavior_configs: vec![(String::from("systray"), || schema_for!(SystrayConfig))],
            styling_configs: vec![],
        }
    }
}

/// Custom icon and color override for tray items matching a pattern.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TrayItemOverride {
    /// Glob pattern to match against item ID or title.
    ///
    /// Examples: `"discord"`, `"*Discord*"`, `"org.kde.*"`
    pub name: String,
    /// Custom icon name (symbolic icon).
    pub icon: Option<String>,
    /// Custom icon color.
    pub color: Option<ColorValue>,
}
