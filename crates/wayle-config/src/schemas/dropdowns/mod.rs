mod custom;

use std::collections::HashMap;

pub use custom::{CustomDropdownDefinition, PickerSectionConfig, SectionDefinition, SectionType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Configuration for dropdown panels.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DropdownsConfig {
    /// Custom user-defined dropdown panels.
    #[serde(default)]
    pub custom: HashMap<String, CustomDropdownDefinition>,
}
