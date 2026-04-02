use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;
use wayle_derive::EnumVariants;

/// Execution mode for custom module commands.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, EnumVariants,
)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionMode {
    /// Run command at regular intervals defined by `interval-ms`.
    ///
    /// Best for commands that complete quickly and return current state
    /// (e.g., reading a file, querying system status).
    #[default]
    Poll,

    /// Spawn long-running process and update display on each stdout line.
    ///
    /// Best for event-driven updates without polling overhead
    /// (e.g., `pactl subscribe`, `inotifywait`, `tail -f`).
    /// Configure `restart-policy` to control restarts after exit.
    Watch,
}

/// Restart behavior for watch-mode custom modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    /// Never restart after exit.
    #[default]
    Never,
    /// Restart after any exit code (success or failure).
    OnExit,
    /// Restart only after non-zero exit codes or signal termination.
    OnFailure,
}

const MIN_MS: u64 = 1;

/// Restart delay in milliseconds, clamped to >= 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct RestartDelay(#[schemars(range(min = MIN_MS))] u64);

impl RestartDelay {
    /// Default delay (1000ms).
    pub const DEFAULT: Self = Self(1000);

    /// Creates a delay, clamping to >= 1ms.
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value.max(MIN_MS))
    }

    /// Returns the inner millisecond value.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }
}

impl Default for RestartDelay {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for RestartDelay {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for RestartDelay {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for RestartDelay {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u64::deserialize(deserializer)?;
        if raw < MIN_MS {
            warn!(
                "restart-interval-ms {} below minimum ({}), clamped",
                raw, MIN_MS
            );
        }
        Ok(Self::new(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_zero_to_minimum() {
        assert_eq!(RestartDelay::new(0).value(), 1);
    }

    #[test]
    fn preserves_valid_value() {
        assert_eq!(RestartDelay::new(500).value(), 500);
    }

    #[test]
    fn default_is_1000ms() {
        assert_eq!(RestartDelay::default().value(), 1000);
    }

    #[test]
    fn deserialize_clamps() {
        let delay: RestartDelay = serde_json::from_str("0").unwrap();
        assert_eq!(delay.value(), 1);
    }

    #[test]
    fn deserialize_valid() {
        let delay: RestartDelay = serde_json::from_str("250").unwrap();
        assert_eq!(delay.value(), 250);
    }
}
