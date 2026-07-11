use schemars::schema_for;
use wayle_derive::{wayle_config, wayle_enum};

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, Percentage, RoundingLevel, Spacing},
};

/// Dock position on screen.
#[wayle_enum]
pub enum DockPosition {
    /// Bottom edge of the screen.
    Bottom,
    /// Left edge of the screen.
    Left,
    /// Right edge of the screen.
    Right,
}

/// Dock visibility mode.
#[wayle_enum(default)]
pub enum DockVisibility {
    /// Dock is always visible.
    #[default]
    AlwaysVisible,
    /// Dock hides automatically and shows on hover.
    Autohide,
}

/// Dock configuration settings.
#[wayle_config(i18n_prefix = "settings-dock")]
pub struct DockConfig {
    /// Dock position: Bottom, Left, or Right.
    #[default(DockPosition::Bottom)]
    pub position: ConfigProperty<DockPosition>,

    /// Visibility mode: always-visible or autohide.
    #[default(DockVisibility::AlwaysVisible)]
    pub visibility: ConfigProperty<DockVisibility>,

    /// Hide threshold for autohide mode in milliseconds.
    #[serde(rename = "autohide-delay")]
    #[default(200u64)]
    pub autohide_delay: ConfigProperty<u64>,

    /// Dock height/width in pixels depending on position.
    #[default(48u32)]
    pub size: ConfigProperty<u32>,

    /// Padding between dock items.
    #[serde(rename = "item-padding")]
    #[default(Spacing::new(0.5))]
    pub item_padding: ConfigProperty<Spacing>,

    /// Corner rounding level for dock items.
    #[serde(rename = "item-rounding")]
    #[default(RoundingLevel::Md)]
    pub item_rounding: ConfigProperty<RoundingLevel>,

    /// Dock background opacity (0-100).
    #[serde(rename = "background-opacity")]
    #[default(Percentage::new(90))]
    pub background_opacity: ConfigProperty<Percentage>,

    /// Dock background color.
    #[default(ColorValue::Token(CssToken::BgSurface))]
    pub bg: ConfigProperty<ColorValue>,

    /// Show running applications alongside pinned.
    #[serde(rename = "show-running")]
    #[default(true)]
    pub show_running: ConfigProperty<bool>,

    /// Enable window preview on hover.
    #[serde(rename = "show-preview")]
    #[default(false)]
    pub show_preview: ConfigProperty<bool>,

    /// Pinned application IDs displayed in the dock.
    #[serde(rename = "pinned-apps")]
    #[default(Vec::<String>::new())]
    pub pinned_apps: ConfigProperty<Vec<String>>,

    /// Active window border width in pixels.
    #[serde(rename = "active-border-width")]
    #[default(2u8)]
    pub active_border_width: ConfigProperty<u8>,

    /// Active window border color.
    #[serde(rename = "active-border-color")]
    #[default(ColorValue::Token(CssToken::Accent))]
    pub active_border_color: ConfigProperty<ColorValue>,
}

impl ModuleInfoProvider for DockConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("dock"),
            schema: || schema_for!(DockConfig),
            layout_id: None,
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        vec![ConfigGroup::general(), ConfigGroup::colors()]
    }
}

crate::register_module!(DockConfig);
