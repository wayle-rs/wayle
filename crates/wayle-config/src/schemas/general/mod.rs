use wayle_derive::wayle_config;

use crate::ConfigProperty;

/// General Wayle configuration.
#[wayle_config(i18n_prefix = "settings-general")]
pub struct GeneralConfig {
    /// Sans-serif font family for UI text and labels.
    #[serde(rename = "font-sans")]
    #[default(String::from("Inter"))]
    pub font_sans: ConfigProperty<String>,

    /// Monospace font family for code and technical content.
    #[serde(rename = "font-mono")]
    #[default(String::from("JetBrains Mono"))]
    pub font_mono: ConfigProperty<String>,

    /// Demote overlay surfaces to allow compositor screen tearing.
    ///
    /// When enabled, surfaces that would normally use the `overlay` layer
    /// are demoted to `top`, allowing fullscreen games to use direct scanout.
    #[serde(rename = "tearing-mode")]
    #[default(false)]
    pub tearing_mode: ConfigProperty<bool>,
}
