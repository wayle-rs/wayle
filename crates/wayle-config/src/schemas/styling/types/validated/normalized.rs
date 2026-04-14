//! Normalized f64 newtype (0.0-1.0).

use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

const MIN: f64 = 0.0;
const MAX: f64 = 1.0;

/// Floating-point value clamped to 0.0-1.0.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct NormalizedF64(#[schemars(range(min = MIN, max = MAX))] f64);

impl NormalizedF64 {
    /// `0.0`
    pub const MIN: f64 = MIN;

    /// `1.0`
    pub const MAX: f64 = MAX;

    /// Creates a normalized value, clamping to 0.0-1.0.
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// The raw `f64`.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

impl Default for NormalizedF64 {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Display for NormalizedF64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f64> for NormalizedF64 {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for NormalizedF64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f64::deserialize(deserializer)?;
        if !(MIN..=MAX).contains(&raw) {
            warn!(
                "normalized value {} out of range (valid: {}-{}), clamped to {}",
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
        assert_eq!(NormalizedF64::new(-0.5).value(), MIN);
        assert_eq!(NormalizedF64::new(-100.0).value(), MIN);
    }

    #[test]
    fn clamps_above_max() {
        assert_eq!(NormalizedF64::new(1.5).value(), MAX);
        assert_eq!(NormalizedF64::new(100.0).value(), MAX);
    }

    #[test]
    fn preserves_valid() {
        assert_eq!(NormalizedF64::new(0.0).value(), 0.0);
        assert_eq!(NormalizedF64::new(0.5).value(), 0.5);
        assert_eq!(NormalizedF64::new(0.77).value(), 0.77);
        assert_eq!(NormalizedF64::new(1.0).value(), 1.0);
    }
}
