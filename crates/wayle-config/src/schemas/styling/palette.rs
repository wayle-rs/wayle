use wayle_derive::wayle_config;

use super::HexColor;
use crate::{ConfigProperty, infrastructure::themes::palettes::wayle_theme};

fn hex(s: &str) -> HexColor {
    HexColor::new(s).unwrap_or_else(|_| HexColor::new(wayle_theme::RED).unwrap_or_default())
}

/// Color palette configuration for the active theme.
#[wayle_config]
pub struct PaletteConfig {
    /// Base background color (darkest).
    #[i18n("settings-palette-bg")]
    #[default(hex(wayle_theme::BG))]
    pub bg: ConfigProperty<HexColor>,

    /// Card and sidebar background.
    #[i18n("settings-palette-surface")]
    #[default(hex(wayle_theme::SURFACE))]
    pub surface: ConfigProperty<HexColor>,

    /// Raised element background.
    #[i18n("settings-palette-elevated")]
    #[default(hex(wayle_theme::ELEVATED))]
    pub elevated: ConfigProperty<HexColor>,

    /// Primary text color.
    #[i18n("settings-palette-fg")]
    #[default(hex(wayle_theme::FG))]
    pub fg: ConfigProperty<HexColor>,

    /// Secondary text color.
    #[serde(rename = "fg-muted")]
    #[i18n("settings-palette-fg-muted")]
    #[default(hex(wayle_theme::FG_MUTED))]
    pub fg_muted: ConfigProperty<HexColor>,

    /// Accent color for interactive elements.
    #[i18n("settings-palette-primary")]
    #[default(hex(wayle_theme::PRIMARY))]
    pub primary: ConfigProperty<HexColor>,

    /// Red semantic color.
    #[i18n("settings-palette-red")]
    #[default(hex(wayle_theme::RED))]
    pub red: ConfigProperty<HexColor>,

    /// Yellow semantic color.
    #[i18n("settings-palette-yellow")]
    #[default(hex(wayle_theme::YELLOW))]
    pub yellow: ConfigProperty<HexColor>,

    /// Green semantic color.
    #[i18n("settings-palette-green")]
    #[default(hex(wayle_theme::GREEN))]
    pub green: ConfigProperty<HexColor>,

    /// Blue semantic color.
    #[i18n("settings-palette-blue")]
    #[default(hex(wayle_theme::BLUE))]
    pub blue: ConfigProperty<HexColor>,
}
