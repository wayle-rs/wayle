use wayle_derive::wayle_config;

use crate::ConfigProperty;

/// General Wayle configuration.
#[wayle_config]
pub struct GeneralConfig {
    /// Sans-serif font family for UI text and labels.
    #[serde(rename = "font-sans")]
    #[i18n("settings-general-font-sans")]
    #[default(String::from("Inter"))]
    pub font_sans: ConfigProperty<String>,

    /// Monospace font family for code and technical content.
    #[serde(rename = "font-mono")]
    #[i18n("settings-general-font-mono")]
    #[default(String::from("JetBrains Mono"))]
    pub font_mono: ConfigProperty<String>,

    /// Demote overlay surfaces to allow compositor screen tearing.
    ///
    /// When enabled, surfaces that would normally use the `overlay` layer
    /// are demoted to `top`, allowing fullscreen games to use direct scanout.
    #[serde(rename = "tearing-mode")]
    #[i18n("settings-general-tearing-mode")]
    #[default(false)]
    pub tearing_mode: ConfigProperty<bool>,
}
