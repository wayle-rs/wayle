//! Cava-specific validated newtypes and enums.

use std::fmt::{self, Display, Formatter};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::warn;

const BAR_COUNT_MIN: u16 = 1;
const BAR_COUNT_MAX: u16 = 256;

/// Frequency bar count clamped to 1-256 (mirrors `wayle_cava::BarCount`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct BarCount(#[schemars(range(min = BAR_COUNT_MIN, max = BAR_COUNT_MAX))] u16);

impl BarCount {
    /// Default bar count.
    pub const DEFAULT: Self = Self(20);

    /// Creates a bar count, clamping to 1-256.
    #[must_use]
    pub fn new(value: u16) -> Self {
        Self(value.clamp(BAR_COUNT_MIN, BAR_COUNT_MAX))
    }

    /// Returns the inner u16 value.
    #[must_use]
    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for BarCount {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for BarCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for BarCount {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for BarCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u16::deserialize(deserializer)?;
        if !(BAR_COUNT_MIN..=BAR_COUNT_MAX).contains(&raw) {
            warn!(
                "bar count {} out of range (valid: {}-{}), clamped to {}",
                raw,
                BAR_COUNT_MIN,
                BAR_COUNT_MAX,
                raw.clamp(BAR_COUNT_MIN, BAR_COUNT_MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

const FRAMERATE_MIN: u32 = 1;
const FRAMERATE_MAX: u32 = 360;

/// Visualization framerate clamped to 1-360 fps (mirrors `wayle_cava::Framerate`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct Framerate(#[schemars(range(min = FRAMERATE_MIN, max = FRAMERATE_MAX))] u32);

impl Framerate {
    /// Default framerate (60 fps).
    pub const DEFAULT: Self = Self(60);

    /// Creates a framerate, clamping to 1-360.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.clamp(FRAMERATE_MIN, FRAMERATE_MAX))
    }

    /// Returns the inner u32 value.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Default for Framerate {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Display for Framerate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for Framerate {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for Framerate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        if !(FRAMERATE_MIN..=FRAMERATE_MAX).contains(&raw) {
            warn!(
                "framerate {} out of range (valid: {}-{}), clamped to {}",
                raw,
                FRAMERATE_MIN,
                FRAMERATE_MAX,
                raw.clamp(FRAMERATE_MIN, FRAMERATE_MAX)
            );
        }
        Ok(Self::new(raw))
    }
}

const FREQUENCY_MIN: u32 = 1;

/// Frequency value in Hz, minimum 1 Hz.
///
/// Cross-field constraints (high_cutoff > low_cutoff, samplerate/2 > high_cutoff)
/// are validated at the service builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct FrequencyHz(#[schemars(range(min = FREQUENCY_MIN))] u32);

impl FrequencyHz {
    /// Creates a frequency value, clamping to >= 1 Hz.
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value.max(FREQUENCY_MIN))
    }

    /// Returns the inner u32 value.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

impl Display for FrequencyHz {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u32> for FrequencyHz {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl<'de> Deserialize<'de> for FrequencyHz {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = u32::deserialize(deserializer)?;
        if raw < FREQUENCY_MIN {
            warn!(
                "frequency {} below minimum ({}), clamped",
                raw, FREQUENCY_MIN
            );
        }
        Ok(Self::new(raw))
    }
}

/// Visualization rendering style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CavaStyle {
    /// Rectangular frequency bars.
    #[default]
    Bars,
    /// Smooth curve connecting bar peaks.
    Wave,
    /// Bars with floating peak indicators that decay over time.
    Peaks,
}

/// Audio capture backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum CavaInput {
    /// PipeWire multimedia server.
    #[default]
    PipeWire,
    /// PulseAudio sound server.
    Pulse,
    /// Advanced Linux Sound Architecture.
    Alsa,
    /// JACK Audio Connection Kit.
    Jack,
    /// Named pipe (FIFO) input.
    Fifo,
    /// PortAudio cross-platform library.
    PortAudio,
    /// sndio audio subsystem (BSD).
    Sndio,
    /// Open Sound System (legacy).
    Oss,
    /// Shared memory input.
    Shmem,
    /// Windows audio capture (WASAPI).
    Winscap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_count_clamps_below_min() {
        assert_eq!(BarCount::new(0).value(), BAR_COUNT_MIN);
    }

    #[test]
    fn bar_count_clamps_above_max() {
        assert_eq!(BarCount::new(BAR_COUNT_MAX + 1).value(), BAR_COUNT_MAX);
    }

    #[test]
    fn bar_count_preserves_valid() {
        assert_eq!(BarCount::new(128).value(), 128);
    }

    #[test]
    fn framerate_clamps_below_min() {
        assert_eq!(Framerate::new(0).value(), FRAMERATE_MIN);
    }

    #[test]
    fn framerate_clamps_above_max() {
        assert_eq!(Framerate::new(FRAMERATE_MAX + 1).value(), FRAMERATE_MAX);
    }

    #[test]
    fn framerate_preserves_valid() {
        assert_eq!(Framerate::new(144).value(), 144);
    }

    #[test]
    fn frequency_clamps_zero_to_min() {
        assert_eq!(FrequencyHz::new(0).value(), FREQUENCY_MIN);
    }

    #[test]
    fn frequency_preserves_valid() {
        assert_eq!(FrequencyHz::new(50).value(), 50);
        assert_eq!(FrequencyHz::new(20000).value(), 20000);
    }
}
