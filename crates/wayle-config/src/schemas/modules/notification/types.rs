use std::{borrow::Cow, fmt};

use serde::{Deserialize, Serialize, Serializer, de};
use wayle_derive::wayle_enum;

/// Screen position for notification popups.
#[wayle_enum(default)]
pub enum PopupPosition {
    /// Top-left corner.
    TopLeft,
    /// Top-center edge.
    TopCenter,
    /// Top-right corner.
    #[default]
    TopRight,
    /// Bottom-left corner.
    BottomLeft,
    /// Bottom-center edge.
    BottomCenter,
    /// Bottom-right corner.
    BottomRight,
    /// Center-left edge.
    CenterLeft,
    /// Center-right edge.
    CenterRight,
}

/// Order in which popups are stacked on screen.
#[wayle_enum(default)]
pub enum StackingOrder {
    /// Newest notifications appear closest to the configured position.
    #[default]
    NewestFirst,
    /// Oldest notifications appear closest to the configured position.
    OldestFirst,
}

/// Behavior when the close button is clicked on a popup card.
#[wayle_enum(default)]
pub enum PopupCloseBehavior {
    /// Hide the popup; notification stays in history.
    #[default]
    Dismiss,
    /// Remove the notification entirely.
    Remove,
}

/// Target monitor for notification popups.
///
/// Accepts `"primary"` or a connector name like `"DP-1"`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum PopupMonitor {
    /// Use the first available monitor (primary).
    #[default]
    Primary,
    /// Use a specific monitor identified by connector name.
    Connector(String),
}

impl Serialize for PopupMonitor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Primary => serializer.serialize_str("primary"),
            Self::Connector(name) => serializer.serialize_str(name),
        }
    }
}

impl<'de> Deserialize<'de> for PopupMonitor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(PopupMonitorVisitor)
    }
}

impl schemars::JsonSchema for PopupMonitor {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("PopupMonitor")
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "description": "\"primary\" or a monitor connector name (e.g. \"DP-1\")",
            "default": "primary"
        })
    }
}

struct PopupMonitorVisitor;

impl de::Visitor<'_> for PopupMonitorVisitor {
    type Value = PopupMonitor;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(r#""primary" or a connector name like "DP-1""#)
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<PopupMonitor, E> {
        if value.eq_ignore_ascii_case("primary") {
            Ok(PopupMonitor::Primary)
        } else {
            Ok(PopupMonitor::Connector(value.to_owned()))
        }
    }
}

/// Minimum urgency level that shows a colored urgency bar on popup cards.
///
/// All urgency levels at or above the threshold display the bar.
/// For example, `Normal` shows bars on both normal and critical popups.
#[wayle_enum(default)]
pub enum UrgencyBarThreshold {
    /// Show urgency bars on all popups.
    #[default]
    Low,
    /// Show urgency bars on normal and critical popups.
    Normal,
    /// Show urgency bars on critical popups only.
    Critical,
    /// Never show urgency bars.
    None,
}

/// Source for resolving notification icons.
#[wayle_enum(default)]
pub enum IconSource {
    /// Use per-notification images when provided, otherwise Wayle's mapped icon.
    #[default]
    Automatic,
    /// Always use Wayle's mapped icons regardless of what the app provides.
    Mapped,
    /// Use the full application icon chain, falling back to mapped if unavailable.
    Application,
}
