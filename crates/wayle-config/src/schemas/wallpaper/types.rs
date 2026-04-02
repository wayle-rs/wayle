use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;
use wayle_derive::EnumVariants;

/// Image scaling mode.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum FitMode {
    /// Scale to cover entire display, cropping excess.
    #[default]
    Fill,
    /// Scale to fit within display, letterboxing if needed.
    Fit,
    /// Display at original size, centered.
    Center,
    /// Stretch to exactly fill, ignoring aspect ratio.
    Stretch,
}

/// Transition animation type.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum TransitionType {
    /// Instant change with no animation.
    None,
    /// Basic crossfade.
    #[default]
    Simple,
    /// Fade with bezier-controlled easing.
    Fade,
    /// Wipe from left edge to right.
    Left,
    /// Wipe from right edge to left.
    Right,
    /// Wipe from top edge to bottom.
    Top,
    /// Wipe from bottom edge to top.
    Bottom,
    /// Wipe at configurable angle.
    Wipe,
    /// Wavy wipe effect.
    Wave,
    /// Growing circle from a position.
    Grow,
    /// Growing circle from center.
    Center,
    /// Shrinking circle from edges inward.
    Outer,
    /// Growing circle from random position.
    Any,
    /// Randomly selects from all transition types.
    Random,
}

/// Wallpaper cycling order.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "lowercase")]
pub enum CyclingMode {
    /// Alphabetical order.
    #[default]
    Sequential,
    /// Random order.
    Shuffle,
}

const DURATION_MIN: f32 = 0.0;

/// Transition duration in seconds, clamped to >= 0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct TransitionDuration(#[schemars(range(min = DURATION_MIN))] f32);

impl TransitionDuration {
    /// Default duration (0.7 seconds).
    pub const DEFAULT: Self = Self(0.7);

    /// Creates a duration, clamping to >= 0.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.max(DURATION_MIN))
    }

    /// Returns the inner f32 value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for TransitionDuration {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for TransitionDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for TransitionDuration {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for TransitionDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f32::deserialize(deserializer)?;
        if raw < DURATION_MIN {
            warn!(
                "transition duration {} below minimum ({}), clamped",
                raw, DURATION_MIN
            );
        }
        Ok(Self::new(raw))
    }
}

const FPS_MIN: u32 = 1;
const FPS_MAX: u32 = 360;

/// Transition frame rate clamped to 1-360 fps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct TransitionFps(#[schemars(range(min = FPS_MIN, max = FPS_MAX))] u32);

impl TransitionFps {
    /// Default frame rate (60 fps).
    pub const DEFAULT: Self = Self(60);

    /// Creates a frame rate, clamping to 1-360.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.clamp(FPS_MIN, FPS_MAX))
    }

    /// Returns the inner u32 value.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for TransitionFps {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for TransitionFps {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for TransitionFps {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for TransitionFps {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        if !(FPS_MIN..=FPS_MAX).contains(&raw) {
            warn!(
                "transition fps {} out of range (valid: {}-{}), clamped to {}",
                raw,
                FPS_MIN,
                FPS_MAX,
                raw.clamp(FPS_MIN, FPS_MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

const INTERVAL_MIN: u64 = 1;

/// Cycling interval in minutes, minimum 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct CyclingInterval(#[schemars(range(min = INTERVAL_MIN))] u64);

impl CyclingInterval {
    /// Default interval (15 minutes).
    pub const DEFAULT: Self = Self(15);

    /// Creates an interval, clamping to >= 1.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value.max(INTERVAL_MIN))
    }

    /// Returns the inner u64 value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for CyclingInterval {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for CyclingInterval {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for CyclingInterval {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for CyclingInterval {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        if raw < INTERVAL_MIN {
            warn!(
                "cycling interval {} below minimum ({}), clamped",
                raw, INTERVAL_MIN
            );
        }
        Ok(Self::new(raw))
    }
}

/// Per-monitor wallpaper configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MonitorWallpaperConfig {
    /// Monitor name (e.g., "HDMI-1", "DP-1").
    pub name: String,
    /// Image scaling mode for this monitor.
    #[serde(default)]
    pub fit_mode: FitMode,
    /// Wallpaper image path for this monitor.
    #[serde(default)]
    pub wallpaper: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transition_duration_clamps_negative() {
        assert_eq!(TransitionDuration::new(-1.0).value(), DURATION_MIN);
    }

    #[test]
    fn transition_duration_preserves_valid() {
        assert!((TransitionDuration::new(1.5).value() - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn transition_fps_clamps_below_min() {
        assert_eq!(TransitionFps::new(0).value(), FPS_MIN);
    }

    #[test]
    fn transition_fps_clamps_above_max() {
        assert_eq!(TransitionFps::new(FPS_MAX + 1).value(), FPS_MAX);
    }

    #[test]
    fn transition_fps_preserves_valid() {
        assert_eq!(TransitionFps::new(144).value(), 144);
    }

    #[test]
    fn cycling_interval_clamps_zero() {
        assert_eq!(CyclingInterval::new(0).value(), INTERVAL_MIN);
    }

    #[test]
    fn cycling_interval_preserves_valid() {
        assert_eq!(CyclingInterval::new(30).value(), 30);
    }
}
