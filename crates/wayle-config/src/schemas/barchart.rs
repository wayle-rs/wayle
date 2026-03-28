//! Generic barchart configuration types.
//!
//! These types are shared across modules that use barchart visualizations
//! (e.g., CAVA audio visualizer, CPU per-core usage).
//!
//! Each module that uses barcharts declares its own configuration properties
//! for bar-width, bar-gap, bar-direction, bar-color, and bar-internal-padding.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Bar growth direction relative to bar's attached screen edge.
///
/// This type is used by modules that display barcharts (CAVA, CPU, etc.).
/// Each module declares its own `bar-direction` config property using this enum.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BarDirection {
    /// Bars grow away from the attached edge.
    #[default]
    Normal,
    /// Bars grow toward the attached edge.
    Reverse,
    /// Bars grow symmetrically from center.
    Mirror,
}
