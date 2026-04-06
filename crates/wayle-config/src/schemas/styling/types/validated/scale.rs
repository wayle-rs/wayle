//! Scale factor newtype.

use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

const MIN: f32 = 0.25;
const MAX: f32 = 3.0;

/// Scale multiplier clamped to 0.25-3.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct ScaleFactor(#[schemars(range(min = MIN, max = MAX))] f32);

impl ScaleFactor {
    /// `0.25`
    pub const MIN: f32 = MIN;

    /// `3.0`
    pub const MAX: f32 = MAX;

    /// Creates a scale factor, clamping to 0.25-3.0.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// The raw `f32`.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Deref for ScaleFactor {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ScaleFactor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for ScaleFactor {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for ScaleFactor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f32::deserialize(deserializer)?;
        if !(MIN..=MAX).contains(&raw) {
            warn!(
                "scale factor {} out of range (valid: {}-{}), clamped to {}",
                raw,
                MIN,
                MAX,
                raw.clamp(MIN, MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_below_min() {
        assert_eq!(ScaleFactor::new(0.1).value(), MIN);
        assert_eq!(ScaleFactor::new(-1.0).value(), MIN);
    }

    #[test]
    fn clamps_above_max() {
        assert_eq!(ScaleFactor::new(5.0).value(), MAX);
    }

    #[test]
    fn preserves_valid() {
        assert_eq!(ScaleFactor::new(1.5).value(), 1.5);
        assert_eq!(ScaleFactor::new(MIN).value(), MIN);
        assert_eq!(ScaleFactor::new(MAX).value(), MAX);
    }
}
