use std::{borrow::Cow, fmt};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer, de};
use wayle_derive::EnumVariants;

/// Screen anchor for the OSD overlay.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum OsdPosition {
    /// Top-left corner.
    TopLeft,
    /// Top-center edge.
    Top,
    /// Top-right corner.
    TopRight,
    /// Right-center edge.
    Right,
    /// Bottom-right corner.
    BottomRight,
    /// Bottom-center edge.
    #[default]
    Bottom,
    /// Bottom-left corner.
    BottomLeft,
    /// Left-center edge.
    Left,
}

/// Target monitor for the OSD overlay.
///
/// Accepts `"primary"` or a connector name like `"DP-1"`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum OsdMonitor {
    /// Use the first available monitor.
    #[default]
    Primary,
    /// Use a specific monitor identified by connector name.
    Connector(String),
}

impl Serialize for OsdMonitor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Primary => serializer.serialize_str("primary"),
            Self::Connector(name) => serializer.serialize_str(name),
        }
    }
}

impl<'de> Deserialize<'de> for OsdMonitor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(OsdMonitorVisitor)
    }
}

impl JsonSchema for OsdMonitor {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("OsdMonitor")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "\"primary\" or a monitor connector name (e.g. \"DP-1\")",
            "default": "primary"
        })
    }
}

struct OsdMonitorVisitor;

impl de::Visitor<'_> for OsdMonitorVisitor {
    type Value = OsdMonitor;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(r#""primary" or a connector name like "DP-1""#)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<OsdMonitor, E> {
        if value.eq_ignore_ascii_case("primary") {
            Ok(OsdMonitor::Primary)
        } else {
            Ok(OsdMonitor::Connector(value.to_owned()))
        }
    }
}
