use tracing::warn;

use super::error::Error;

/// Multi-channel volume with safety warnings
///
/// # Volume Safety Guidelines
/// - **Safe range**: 0.0 to 2.0 (0% to 200%)
/// - **Warning range**: 2.0 to 4.0 (may cause audio damage)
/// - **Invalid**: Above 4.0 (clamped) or below 0.0 (clamped)
///
/// # Volume Levels
/// - 0.0 = Muted
/// - 1.0 = Normal volume (100%)
/// - 2.0 = Safe maximum (200%)
/// - 4.0 = Absolute maximum (400% - **Audio damage possible**)
#[derive(Debug, Clone, PartialEq)]
pub struct Volume {
    volumes: Vec<f64>,
}

impl Volume {
    /// Create a new volume with the given channel volumes
    ///
    /// Volume levels are automatically clamped to valid range (0.0 to 4.0).
    /// - 0.0 = Muted
    /// - 1.0 = Normal volume (100%)
    /// - 4.0 = Maximum amplification (400%)
    pub fn new(volumes: Vec<f64>) -> Self {
        let volumes = volumes.into_iter().map(|v| {
            let clamped = v.clamp(0.0, 4.0);
            if v > 2.0 && v <= 4.0 {
                warn!("Volume {v} exceeds safe limit (2.0). Audio damage possible at high amplification.");
            } else if v > 4.0 {
                warn!("Volume {v} clamped to maximum (4.0). Use values ≤2.0 for safe operation.");
            } else if v < 0.0 {
                warn!("Negative volume {v} clamped to 0.0.");
            }
            clamped
        }).collect();
        Self { volumes }
    }

    /// Creates volume with amplification (allows up to 4.0).
    ///
    /// Unlike `new()`, this method validates input and returns an error
    /// for out-of-range values instead of clamping.
    ///
    /// Note: Volumes above 2.0 may cause audio damage or distortion.
    ///
    /// # Errors
    /// Returns error if any volume is negative or exceeds 4.0.
    pub fn with_amplification(volumes: Vec<f64>) -> Result<Self, Error> {
        for (channel, &volume) in volumes.iter().enumerate() {
            if !(0.0..=4.0).contains(&volume) {
                return Err(Error::InvalidVolume { channel, volume });
            }
        }
        Ok(Self { volumes })
    }

    /// Creates a mono (single-channel) volume.
    ///
    /// Volume is automatically clamped to valid range (0.0 to 4.0).
    /// A value of 1.0 represents normal volume, and values above 1.0 provide amplification.
    pub fn mono(volume: f64) -> Self {
        Self::new(vec![volume])
    }

    /// Creates a stereo (two-channel) volume with left and right levels.
    ///
    /// Volume levels are automatically clamped to valid range (0.0 to 4.0).
    /// A value of 1.0 represents normal volume, and values above 1.0 provide amplification.
    pub fn stereo(left: f64, right: f64) -> Self {
        Self::new(vec![left, right])
    }

    /// Get volume for a specific channel
    pub fn channel(&self, channel: usize) -> Option<f64> {
        self.volumes.get(channel).copied()
    }

    /// Set volume for a specific channel
    ///
    /// Volume is automatically clamped to valid range (0.0 to 4.0).
    ///
    /// # Errors
    /// Returns error if channel index is out of bounds.
    #[allow(clippy::cognitive_complexity)]
    pub fn set_channel(&mut self, channel: usize, volume: f64) -> Result<(), Error> {
        if let Some(vol) = self.volumes.get_mut(channel) {
            let clamped = volume.clamp(0.0, 4.0);
            if volume > 2.0 && volume <= 4.0 {
                warn!(
                    "Volume {volume} exceeds safe limit (2.0). Audio damage possible at high amplification."
                );
            } else if volume > 4.0 {
                warn!(
                    "Volume {volume} clamped to maximum (4.0). Use values ≤2.0 for safe operation."
                );
            } else if volume < 0.0 {
                warn!("Negative volume {volume} clamped to 0.0.");
            }
            *vol = clamped;
            Ok(())
        } else {
            Err(Error::InvalidChannel { channel })
        }
    }

    /// Get average volume across all channels
    pub fn average(&self) -> f64 {
        if self.volumes.is_empty() {
            0.0
        } else {
            self.volumes.iter().sum::<f64>() / self.volumes.len() as f64
        }
    }

    /// Get number of channels
    pub fn channels(&self) -> usize {
        self.volumes.len()
    }

    /// Get all channel volumes
    pub fn as_slice(&self) -> &[f64] {
        &self.volumes
    }

    /// Create a muted volume (0.0)
    pub fn muted(channels: usize) -> Self {
        Self::new(vec![0.0; channels])
    }

    /// Create a normal volume (1.0 = 100%)
    pub fn normal(channels: usize) -> Self {
        Self::new(vec![1.0; channels])
    }

    /// Create a volume from percentage (0-100% maps to 0.0-1.0)
    pub fn from_percentage(percentage: f64, channels: usize) -> Self {
        let volume = percentage / 100.0;
        Self::new(vec![volume; channels])
    }

    /// Get volume as percentage (1.0 = 100%)
    pub fn to_percentage(&self) -> Vec<f64> {
        self.volumes.iter().map(|&v| v * 100.0).collect()
    }

    /// Get average volume as percentage (1.0 = 100%).
    pub fn average_percentage(&self) -> f64 {
        self.average() * 100.0
    }

    /// Check if volume is muted (all channels at 0.0)
    pub fn is_muted(&self) -> bool {
        self.volumes.iter().all(|&v| v == 0.0)
    }

    /// Check if volume is at normal level (all channels at 1.0)
    pub fn is_normal(&self) -> bool {
        self.volumes.iter().all(|&v| v == 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_negative_volumes_to_zero() {
        let volume = Volume::new(vec![-1.0, -0.5]);

        assert_eq!(volume.as_slice(), &[0.0, 0.0]);
    }

    #[test]
    fn new_clamps_volumes_above_four_to_four() {
        let volume = Volume::new(vec![5.0, 10.0]);

        assert_eq!(volume.as_slice(), &[4.0, 4.0]);
    }

    #[test]
    fn new_preserves_volumes_in_valid_range() {
        let volume = Volume::new(vec![0.5, 1.0, 2.0, 3.5]);

        assert_eq!(volume.as_slice(), &[0.5, 1.0, 2.0, 3.5]);
    }

    #[test]
    fn mono_creates_single_channel_volume() {
        let volume = Volume::mono(1.5);

        assert_eq!(volume.channels(), 1);
        assert_eq!(volume.channel(0), Some(1.5));
    }

    #[test]
    fn stereo_creates_two_channel_volume() {
        let volume = Volume::stereo(0.8, 1.2);

        assert_eq!(volume.channels(), 2);
        assert_eq!(volume.channel(0), Some(0.8));
        assert_eq!(volume.channel(1), Some(1.2));
    }

    #[test]
    fn with_amplification_accepts_valid_range() {
        let result = Volume::with_amplification(vec![0.0, 2.0, 4.0]);

        assert!(result.is_ok());
        let volume = result.unwrap();
        assert_eq!(volume.as_slice(), &[0.0, 2.0, 4.0]);
    }

    #[test]
    fn with_amplification_rejects_negative_volume() {
        let result = Volume::with_amplification(vec![-0.1]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidVolume { channel: 0, .. }
        ));
    }

    #[test]
    fn with_amplification_rejects_volume_above_four() {
        let result = Volume::with_amplification(vec![4.1]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidVolume { channel: 0, .. }
        ));
    }

    #[test]
    fn set_channel_updates_valid_channel() {
        let mut volume = Volume::stereo(1.0, 1.0);

        let result = volume.set_channel(0, 0.5);

        assert!(result.is_ok());
        assert_eq!(volume.channel(0), Some(0.5));
        assert_eq!(volume.channel(1), Some(1.0));
    }

    #[test]
    fn set_channel_clamps_out_of_range_values() {
        let mut volume = Volume::mono(1.0);

        volume.set_channel(0, -1.0).ok();
        assert_eq!(volume.channel(0), Some(0.0));

        volume.set_channel(0, 5.0).ok();
        assert_eq!(volume.channel(0), Some(4.0));
    }

    #[test]
    fn set_channel_returns_error_for_invalid_channel() {
        let mut volume = Volume::mono(1.0);

        let result = volume.set_channel(5, 1.0);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::InvalidChannel { channel: 5 }
        ));
    }

    #[test]
    fn channel_returns_volume_for_valid_index() {
        let volume = Volume::new(vec![0.5, 1.0, 1.5]);

        assert_eq!(volume.channel(0), Some(0.5));
        assert_eq!(volume.channel(1), Some(1.0));
        assert_eq!(volume.channel(2), Some(1.5));
    }

    #[test]
    fn channel_returns_none_for_invalid_index() {
        let volume = Volume::mono(1.0);

        assert_eq!(volume.channel(1), None);
        assert_eq!(volume.channel(100), None);
    }

    #[test]
    fn average_returns_zero_for_empty_channels() {
        let volume = Volume::new(vec![]);

        assert_eq!(volume.average(), 0.0);
    }

    #[test]
    fn average_calculates_correct_value_for_multiple_channels() {
        let volume = Volume::new(vec![0.5, 1.0, 1.5, 2.0]);

        let avg = volume.average();

        assert!((avg - 1.25).abs() < 0.01);
    }

    #[test]
    fn is_muted_returns_true_when_all_channels_zero() {
        let volume = Volume::new(vec![0.0, 0.0, 0.0]);

        assert!(volume.is_muted());
    }

    #[test]
    fn is_muted_returns_false_when_any_channel_nonzero() {
        let volume = Volume::new(vec![0.0, 0.1, 0.0]);

        assert!(!volume.is_muted());
    }

    #[test]
    fn is_normal_returns_true_when_all_channels_one() {
        let volume = Volume::new(vec![1.0, 1.0, 1.0]);

        assert!(volume.is_normal());
    }

    #[test]
    fn is_normal_returns_false_when_any_channel_not_one() {
        let volume = Volume::new(vec![1.0, 0.9, 1.0]);

        assert!(!volume.is_normal());
    }

    #[test]
    fn from_percentage_converts_correctly() {
        let volume = Volume::from_percentage(50.0, 2);

        assert_eq!(volume.channels(), 2);
        assert_eq!(volume.channel(0), Some(0.5));
        assert_eq!(volume.channel(1), Some(0.5));
    }

    #[test]
    fn to_percentage_converts_correctly() {
        let volume = Volume::new(vec![0.5, 1.0, 1.5]);

        let percentages = volume.to_percentage();

        assert_eq!(percentages, vec![50.0, 100.0, 150.0]);
    }

    #[test]
    fn average_percentage_returns_average_as_percentage() {
        let volume = Volume::stereo(0.5, 1.0);

        let avg_pct = volume.average_percentage();

        assert!((avg_pct - 75.0).abs() < 0.01);
    }
}
