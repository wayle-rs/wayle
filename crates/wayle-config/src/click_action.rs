//! Click action types for bar module interaction.

use serde::{Deserialize, Serialize};

/// Action to perform on a bar module click or scroll event.
///
/// Serializes to/from a string for TOML config compatibility:
/// - `""` -> `None`
/// - `"dropdown"` -> `InlineDropdown`
/// - `"dropdown:audio"` -> `Dropdown("audio")`
/// - `"pavucontrol"` -> `Shell("pavucontrol")`
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ClickAction {
    /// Open a named dropdown panel.
    Dropdown(String),
    /// Open the module's own inline dropdown (custom modules only).
    InlineDropdown,
    /// Execute a shell command.
    Shell(String),
    #[default]
    /// No action configured.
    None,
}

impl ClickAction {
    /// Parse a click action from a config string value.
    pub fn parse_str(s: &str) -> Self {
        if s.is_empty() {
            return Self::None;
        }
        if s == "dropdown" {
            return Self::InlineDropdown;
        }
        match s.strip_prefix("dropdown:") {
            Some(name) => Self::Dropdown(name.to_owned()),
            None => Self::Shell(s.to_owned()),
        }
    }
}

impl Serialize for ClickAction {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Dropdown(name) => serializer.serialize_str(&format!("dropdown:{name}")),
            Self::InlineDropdown => serializer.serialize_str("dropdown"),
            Self::Shell(cmd) => serializer.serialize_str(cmd),
            Self::None => serializer.serialize_str(""),
        }
    }
}

impl<'de> Deserialize<'de> for ClickAction {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Ok(Self::parse_str(&s))
    }
}

impl schemars::JsonSchema for ClickAction {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ClickAction")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}
