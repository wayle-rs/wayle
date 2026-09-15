use libpulse_binding::volume::{ChannelVolumes, Volume as PulseVolume};

use crate::volume::types::Volume;

pub(crate) fn to_pulse(volume: &Volume) -> ChannelVolumes {
    let channels = volume.channels();
    if channels == 0 {
        return ChannelVolumes::default();
    }

    let mut pulse_volume = ChannelVolumes::default();
    pulse_volume.set(channels as u8, PulseVolume::MUTED);

    for (channel_idx, &level) in volume.as_slice().iter().enumerate() {
        pulse_volume.get_mut()[channel_idx] =
            PulseVolume((level * PulseVolume::NORMAL.0 as f64) as u32);
    }

    pulse_volume
}

pub(crate) fn from_pulse(pulse_volume: &ChannelVolumes) -> Volume {
    let volumes: Vec<f64> = (0..pulse_volume.len())
        .map(|channel_idx| {
            let raw = pulse_volume.get()[channel_idx as usize].0 as f64;
            raw / PulseVolume::NORMAL.0 as f64
        })
        .collect();

    Volume::new(volumes)
}

pub(super) fn from_pulse_single(pulse_volume: PulseVolume) -> Volume {
    Volume::new(vec![pulse_volume.0 as f64 / PulseVolume::NORMAL.0 as f64])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_channels_returns_default() {
        let volume = Volume::new(vec![]);

        let result = to_pulse(&volume);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn preserves_per_channel_values() {
        let volume = Volume::stereo(0.3, 0.8);

        let result = to_pulse(&volume);

        assert_eq!(result.len(), 2);
        let left = result.get()[0].0 as f64 / PulseVolume::NORMAL.0 as f64;
        let right = result.get()[1].0 as f64 / PulseVolume::NORMAL.0 as f64;
        assert!((left - 0.3).abs() < 0.001);
        assert!((right - 0.8).abs() < 0.001);
    }

    #[test]
    fn normal_volume_maps_to_pa_normal() {
        let volume = Volume::mono(1.0);

        let result = to_pulse(&volume);

        assert_eq!(result.len(), 1);
        assert_eq!(result.avg(), PulseVolume::NORMAL);
    }

    #[test]
    fn zero_volume_maps_to_muted() {
        let volume = Volume::mono(0.0);

        let result = to_pulse(&volume);

        assert_eq!(result.len(), 1);
        assert_eq!(result.avg(), PulseVolume::MUTED);
    }

    #[test]
    fn max_volume_maps_correctly() {
        let volume = Volume::mono(4.0);

        let result = to_pulse(&volume);

        let expected = PulseVolume((4.0 * PulseVolume::NORMAL.0 as f64) as u32);
        assert_eq!(result.avg(), expected);
    }

    #[test]
    fn from_pulse_normal_volume() {
        let mut pulse_volume = ChannelVolumes::default();
        pulse_volume.set(1, PulseVolume::NORMAL);

        let result = from_pulse(&pulse_volume);

        assert_eq!(result.channels(), 1);
        assert_eq!(result.channel(0), Some(1.0));
    }

    #[test]
    fn from_pulse_muted() {
        let mut pulse_volume = ChannelVolumes::default();
        pulse_volume.set(1, PulseVolume::MUTED);

        let result = from_pulse(&pulse_volume);

        assert_eq!(result.channels(), 1);
        assert_eq!(result.channel(0), Some(0.0));
    }

    #[test]
    fn from_pulse_max_volume() {
        let mut pulse_volume = ChannelVolumes::default();
        let max = PulseVolume((4.0 * PulseVolume::NORMAL.0 as f64) as u32);
        pulse_volume.set(1, max);

        let result = from_pulse(&pulse_volume);

        assert_eq!(result.channels(), 1);
        assert!((result.channel(0).unwrap() - 4.0).abs() < 0.01);
    }

    #[test]
    fn from_pulse_multi_channel() {
        let mut pulse_volume = ChannelVolumes::default();
        pulse_volume.set(2, PulseVolume::NORMAL);

        let result = from_pulse(&pulse_volume);

        assert_eq!(result.channels(), 2);
        assert_eq!(result.channel(0), Some(1.0));
        assert_eq!(result.channel(1), Some(1.0));
    }

    #[test]
    fn roundtrip_preserves_stereo_balance() {
        let original = Volume::stereo(0.3, 0.8);

        let pulse = to_pulse(&original);
        let roundtrip = from_pulse(&pulse);

        assert_eq!(roundtrip.channels(), 2);
        assert!((roundtrip.channel(0).unwrap() - 0.3).abs() < 0.001);
        assert!((roundtrip.channel(1).unwrap() - 0.8).abs() < 0.001);
    }
}
