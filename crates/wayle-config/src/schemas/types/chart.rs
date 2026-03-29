//! Generic chart configuration types.
//!
//! These types are shared across modules that use chart visualizations
//! such as waves, barcharts, etc

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Visualization growth direction relative to attached screen edge.
///
/// This type is used by modules that display charts (CAVA, CPU, etc.).
/// Each module declares its own `direction` config property using this enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// Visualizations grow away from the attached edge.
    #[default]
    Normal,
    /// Visualizations grow toward the attached edge.
    Reverse,
    /// Visualizations grow symmetrically from center.
    Mirror,
}
