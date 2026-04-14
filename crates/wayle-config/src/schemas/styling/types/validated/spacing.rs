//! Non-negative spacing newtype.

use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

const MIN: f32 = 0.0;

/// Non-negative spacing value (clamped at 0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct Spacing(#[schemars(range(min = MIN))] f32);

impl Spacing {
    /// `0.0`
    pub const MIN: f32 = MIN;

    /// Creates a spacing value, clamping negatives to 0.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.max(Self::MIN))
    }

    /// The raw `f32`.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Default for Spacing {
    fn default() -> Self {
        Self(0.0)
    }
}

impl Deref for Spacing {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Spacing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for Spacing {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for Spacing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = f32::deserialize(deserializer)?;
        if raw < MIN {
            warn!("spacing {} cannot be negative, clamped to 0", raw);
        }
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_negative() {
        assert_eq!(Spacing::new(-1.0).value(), 0.0);
        assert_eq!(Spacing::new(-100.0).value(), 0.0);
    }

    #[test]
    fn preserves_non_negative() {
        assert_eq!(Spacing::new(0.0).value(), 0.0);
        assert_eq!(Spacing::new(1.5).value(), 1.5);
        assert_eq!(Spacing::new(100.0).value(), 100.0);
    }
}
