//! Generic chart rendering parameters shared across charting primitives.

use wayle_config::schemas::types::chart::Direction;

/// RGBA color with components normalized to 0.0-1.0.
#[derive(Clone,Copy)]
pub struct Rgba {
    /// Red component (0.0-1.0).
    pub red: f64,
    /// Green component (0.0-1.0).
    pub green: f64,
    /// Blue component (0.0-1.0).
    pub blue: f64,
    /// Alpha/opacity component (0.0-1.0).
    pub alpha: f64,
}

/// Generic rendering parameters shared across visualizations.
#[derive(Clone)]
pub struct Params {
    /// Fill color for the visualization.
    pub fill_color: Rgba,
    /// canvas height
    pub height: f64,
    /// Direction of growth of the visualization relative to attached screen edge
    pub direction: Direction,
}
