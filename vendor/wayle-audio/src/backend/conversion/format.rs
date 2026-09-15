use libpulse_binding::{
    channelmap::{Map as PulseChannelMap, Position},
    sample::{Format as PulseFormat, Spec as PulseSampleSpec},
};

use crate::types::format::{ChannelMap, ChannelPosition, SampleFormat, SampleSpec};

pub(crate) fn convert_sample_format(format: PulseFormat) -> SampleFormat {
    match format {
        PulseFormat::U8 => SampleFormat::U8,
        PulseFormat::ALaw => SampleFormat::ALaw,
        PulseFormat::ULaw => SampleFormat::ULaw,
        PulseFormat::S16le => SampleFormat::S16LE,
        PulseFormat::S16be => SampleFormat::S16BE,
        PulseFormat::S24le => SampleFormat::S24LE,
        PulseFormat::S24be => SampleFormat::S24BE,
        PulseFormat::S24_32le => SampleFormat::S24_32LE,
        PulseFormat::S24_32be => SampleFormat::S24_32BE,
        PulseFormat::S32le => SampleFormat::S32LE,
        PulseFormat::S32be => SampleFormat::S32BE,
        PulseFormat::F32le => SampleFormat::F32LE,
        PulseFormat::F32be => SampleFormat::F32BE,
        _ => SampleFormat::Unknown,
    }
}

pub(super) fn convert_channel_map(pulse_map: &PulseChannelMap) -> ChannelMap {
    let positions = pulse_map
        .get()
        .iter()
        .take(pulse_map.len() as usize)
        .map(|&position| convert_channel_position(position))
        .collect();

    ChannelMap {
        channels: pulse_map.len(),
        positions,
    }
}

pub(super) fn convert_sample_spec(spec: &PulseSampleSpec) -> SampleSpec {
    SampleSpec {
        format: convert_sample_format(spec.format),
        rate: spec.rate,
        channels: spec.channels,
    }
}

fn convert_channel_position(position: Position) -> ChannelPosition {
    match position {
        Position::Mono => ChannelPosition::Mono,
        Position::FrontLeft => ChannelPosition::FrontLeft,
        Position::FrontRight => ChannelPosition::FrontRight,
        Position::FrontCenter => ChannelPosition::FrontCenter,
        Position::RearLeft => ChannelPosition::RearLeft,
        Position::RearRight => ChannelPosition::RearRight,
        Position::RearCenter => ChannelPosition::RearCenter,
        Position::Lfe => ChannelPosition::LFE,
        Position::SideLeft => ChannelPosition::SideLeft,
        Position::SideRight => ChannelPosition::SideRight,
        Position::FrontLeftOfCenter => ChannelPosition::FrontLeftOfCenter,
        Position::FrontRightOfCenter => ChannelPosition::FrontRightOfCenter,
        Position::TopCenter => ChannelPosition::TopCenter,
        Position::TopFrontLeft => ChannelPosition::TopFrontLeft,
        Position::TopFrontRight => ChannelPosition::TopFrontRight,
        Position::TopFrontCenter => ChannelPosition::TopFrontCenter,
        Position::TopRearLeft => ChannelPosition::TopRearLeft,
        Position::TopRearRight => ChannelPosition::TopRearRight,
        Position::TopRearCenter => ChannelPosition::TopRearCenter,
        _ => ChannelPosition::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_format() {
        assert_eq!(convert_sample_format(PulseFormat::U8), SampleFormat::U8);
    }

    #[test]
    fn s16le_format() {
        assert_eq!(
            convert_sample_format(PulseFormat::S16le),
            SampleFormat::S16LE
        );
    }

    #[test]
    fn s16be_format() {
        assert_eq!(
            convert_sample_format(PulseFormat::S16be),
            SampleFormat::S16BE
        );
    }

    #[test]
    fn invalid_format_maps_to_unknown() {
        assert_eq!(
            convert_sample_format(PulseFormat::Invalid),
            SampleFormat::Unknown
        );
    }
}
