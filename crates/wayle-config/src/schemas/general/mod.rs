use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ConfigGroup, GroupDefaults, ModuleInfo, ModuleInfoProvider},
};

/// Shell-wide settings that don't belong to any specific module.
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

impl ModuleInfoProvider for GeneralConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("general"),
            schema: || schema_for!(GeneralConfig),
            layout_id: None,
            array_entry: false,
        }
    }

    fn groups() -> Vec<ConfigGroup> {
        GroupDefaults::standard()
    }
}

crate::register_module!(GeneralConfig);
