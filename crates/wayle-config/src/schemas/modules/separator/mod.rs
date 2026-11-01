use schemars::schema_for;
use wayle_derive::wayle_config;

use crate::{
    ConfigProperty,
    docs::{ModuleInfo, ModuleInfoProvider},
    schemas::styling::{ColorValue, CssToken, Spacing},
};

/// A vertical rule between bar modules.
#[wayle_config(i18n_prefix = "settings-modules-separator")]
pub struct SeparatorConfig {
    /// Thickness of the separator line in pixels.
    #[serde(rename = "size")]
    #[default(1)]
    pub size: ConfigProperty<u32>,

    /// Length of the separator line.
    #[serde(rename = "length")]
    #[default(Spacing::new(1.5))]
    pub length: ConfigProperty<Spacing>,

    /// Color of the separator line.
    #[serde(rename = "color")]
    #[default(ColorValue::Token(CssToken::FgSubtle))]
    pub color: ConfigProperty<ColorValue>,
}

impl ModuleInfoProvider for SeparatorConfig {
    fn module_info() -> ModuleInfo {
        ModuleInfo {
            name: String::from("separator"),
            schema: || schema_for!(SeparatorConfig),
            layout_id: Some(String::from("separator")),
            array_entry: false,
        }
    }
}

crate::register_module!(SeparatorConfig);
