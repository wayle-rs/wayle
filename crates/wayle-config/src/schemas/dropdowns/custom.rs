use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Definition of a custom dropdown panel with composable sections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CustomDropdownDefinition {
    /// Optional title displayed in the dropdown header.
    #[serde(default)]
    pub title: Option<String>,

    /// Optional icon name for the dropdown header.
    #[serde(default)]
    pub icon: Option<String>,

    /// Base width in pixels (scaled by bar scale factor).
    #[serde(default = "default_width")]
    pub width: f32,

    /// Composable sections that make up the dropdown content.
    #[serde(default)]
    pub sections: Vec<SectionDefinition>,
}

/// A section within a custom dropdown.
///
/// The `section_type` field discriminates which other fields are relevant.
/// Fields for inactive section types are ignored at runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SectionDefinition {
    /// The type of section to render.
    #[serde(rename = "type")]
    pub section_type: SectionType,

    /// Shell command that outputs the list of selectable items.
    /// Used when `type = "picker"`.
    ///
    /// The command should output either:
    /// - Plain text (one item per line)
    /// - JSON lines: `{"value": "...", "label": "...", "subtitle": "...", "icon": "...", "active": true}`
    #[serde(default)]
    pub list_command: Option<String>,

    /// Shell command to run when an item is selected.
    /// Used when `type = "picker"`.
    ///
    /// The selected value is available as `$WAYLE_SELECTED`.
    #[serde(default)]
    pub select_command: Option<String>,
}

/// Available section types for custom dropdowns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SectionType {
    /// List-based item picker driven by shell commands.
    Picker,
}

/// Configuration for a picker section (extracted view of picker-specific fields).
///
/// This provides a typed view over the flat fields on [`SectionDefinition`]
/// for convenience when building the picker UI.
#[derive(Debug, Clone)]
pub struct PickerSectionConfig {
    /// Shell command that outputs the list of selectable items.
    pub list_command: Option<String>,

    /// Shell command to run when an item is selected.
    pub select_command: Option<String>,
}

impl SectionDefinition {
    /// Extract picker-specific configuration from this section.
    ///
    /// Returns `Some` only when `section_type` is [`SectionType::Picker`].
    pub fn as_picker(&self) -> Option<PickerSectionConfig> {
        match self.section_type {
            SectionType::Picker => Some(PickerSectionConfig {
                list_command: self.list_command.clone(),
                select_command: self.select_command.clone(),
            }),
        }
    }
}

fn default_width() -> f32 {
    320.0
}
