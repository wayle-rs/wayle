use libpulse_binding::{
    context::introspect::{SinkInfo as PulseSinkInfo, SourceInfo as PulseSourceInfo},
    def::{SinkState, SourceState},
};

use super::{
    format::{convert_channel_map, convert_sample_spec},
    pulse::{active_port_name, collect_proplist, convert_formats, convert_ports, cow_to_string},
    volume::{from_pulse, from_pulse_single},
};
use crate::types::device::{DeviceInfo, DeviceState, SinkInfo, SourceInfo};

pub(crate) fn from_sink(sink: &PulseSinkInfo) -> SinkInfo {
    SinkInfo {
        device: DeviceInfo {
            index: sink.index,
            name: cow_to_string(sink.name.as_ref()),
            description: cow_to_string(sink.description.as_ref()),
            card_index: sink.card,
            owner_module: sink.owner_module,
            driver: cow_to_string(sink.driver.as_ref()),
            state: convert_sink_state(sink.state),
            volume: from_pulse(&sink.volume),
            base_volume: from_pulse_single(sink.base_volume),
            n_volume_steps: sink.n_volume_steps,
            muted: sink.mute,
            properties: collect_proplist(&sink.proplist),
            ports: convert_ports(&sink.ports),
            active_port: active_port_name(&sink.active_port),
            formats: convert_formats(&sink.formats),
            sample_spec: convert_sample_spec(&sink.sample_spec),
            channel_map: convert_channel_map(&sink.channel_map),
            latency: sink.latency,
            configured_latency: sink.configured_latency,
            flags: sink.flags.bits(),
        },
        monitor_source: sink.monitor_source,
        monitor_source_name: cow_to_string(sink.monitor_source_name.as_ref()),
    }
}

pub(crate) fn from_source(source: &PulseSourceInfo) -> SourceInfo {
    SourceInfo {
        device: DeviceInfo {
            index: source.index,
            name: cow_to_string(source.name.as_ref()),
            description: cow_to_string(source.description.as_ref()),
            card_index: source.card,
            owner_module: source.owner_module,
            driver: cow_to_string(source.driver.as_ref()),
            state: convert_source_state(source.state),
            volume: from_pulse(&source.volume),
            base_volume: from_pulse_single(source.base_volume),
            n_volume_steps: source.n_volume_steps,
            muted: source.mute,
            properties: collect_proplist(&source.proplist),
            ports: convert_ports(&source.ports),
            active_port: active_port_name(&source.active_port),
            formats: convert_formats(&source.formats),
            sample_spec: convert_sample_spec(&source.sample_spec),
            channel_map: convert_channel_map(&source.channel_map),
            latency: source.latency,
            configured_latency: source.configured_latency,
            flags: source.flags.bits(),
        },
        monitor_of_sink: source.monitor_of_sink,
        monitor_of_sink_name: source
            .monitor_of_sink
            .map(|_| cow_to_string(source.monitor_of_sink_name.as_ref())),
        is_monitor: source.monitor_of_sink.is_some(),
    }
}

fn convert_sink_state(state: SinkState) -> DeviceState {
    match state {
        SinkState::Running => DeviceState::Running,
        SinkState::Idle => DeviceState::Idle,
        SinkState::Suspended => DeviceState::Suspended,
        _ => DeviceState::Offline,
    }
}

fn convert_source_state(state: SourceState) -> DeviceState {
    match state {
        SourceState::Running => DeviceState::Running,
        SourceState::Idle => DeviceState::Idle,
        SourceState::Suspended => DeviceState::Suspended,
        _ => DeviceState::Offline,
    }
}

#[cfg(test)]
mod tests {
    use libpulse_binding::{
        sample::Format as PulseFormat,
        volume::{ChannelVolumes, Volume as PulseVolume},
    };

    use super::*;

    #[test]
    fn sink_running_state() {
        let sink = create_minimal_sink(SinkState::Running);
        assert_eq!(from_sink(&sink).device.state, DeviceState::Running);
    }

    #[test]
    fn sink_idle_state() {
        let sink = create_minimal_sink(SinkState::Idle);
        assert_eq!(from_sink(&sink).device.state, DeviceState::Idle);
    }

    #[test]
    fn sink_suspended_state() {
        let sink = create_minimal_sink(SinkState::Suspended);
        assert_eq!(from_sink(&sink).device.state, DeviceState::Suspended);
    }

    fn create_minimal_sink(state: SinkState) -> PulseSinkInfo<'static> {
        let mut channel_map = libpulse_binding::channelmap::Map::default();
        channel_map.init_stereo();

        PulseSinkInfo {
            name: Some("test-sink".into()),
            index: 0,
            description: Some("Test Sink".into()),
            sample_spec: libpulse_binding::sample::Spec {
                format: PulseFormat::S16le,
                channels: 2,
                rate: 44100,
            },
            channel_map,
            owner_module: None,
            volume: ChannelVolumes::default(),
            mute: false,
            monitor_source: 1,
            monitor_source_name: Some("test-sink.monitor".into()),
            latency: libpulse_binding::time::MicroSeconds(0),
            driver: Some("test-driver".into()),
            flags: libpulse_binding::def::SinkFlagSet::empty(),
            proplist: libpulse_binding::proplist::Proplist::new().unwrap(),
            configured_latency: libpulse_binding::time::MicroSeconds(0),
            base_volume: PulseVolume::NORMAL,
            state,
            n_volume_steps: 65536,
            card: None,
            ports: vec![],
            active_port: None,
            formats: vec![],
        }
    }
}
