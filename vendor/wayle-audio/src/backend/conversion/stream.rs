use libpulse_binding::context::introspect::{SinkInputInfo, SourceOutputInfo};

use super::{
    format::{convert_channel_map, convert_sample_spec},
    pulse::{collect_proplist, extract_media_info},
    volume::from_pulse,
};
use crate::types::stream::{StreamInfo, StreamState, StreamType};

pub(crate) fn from_sink_input(input: &SinkInputInfo) -> StreamInfo {
    StreamInfo {
        index: input.index,
        stream_type: StreamType::Playback,
        name: input.name.clone().unwrap_or_default().to_string(),
        application_name: input.proplist.get_str("application.name"),
        binary: input.proplist.get_str("application.process.binary"),
        pid: input
            .proplist
            .get_str("application.process.id")
            .and_then(|pid_str| pid_str.parse::<u32>().ok()),
        owner_module: input.owner_module,
        client: input.client,
        device_index: input.sink,
        volume: from_pulse(&input.volume),
        muted: input.mute,
        corked: input.corked,
        has_volume: input.has_volume,
        volume_writable: input.volume_writable,
        state: if input.corked {
            StreamState::Corked
        } else {
            StreamState::Running
        },
        sample_spec: convert_sample_spec(&input.sample_spec),
        channel_map: convert_channel_map(&input.channel_map),
        properties: collect_proplist(&input.proplist),
        media: extract_media_info(&input.proplist),
        buffer_latency: input.buffer_usec.0,
        device_latency: input.sink_usec.0,
        resample_method: input.resample_method.as_ref().map(|s| s.to_string()),
        driver: input
            .driver
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default(),
        format: Some(format!("{:?}", input.format.get_encoding())),
    }
}

pub(crate) fn from_source_output(output: &SourceOutputInfo) -> StreamInfo {
    StreamInfo {
        index: output.index,
        stream_type: StreamType::Record,
        name: output.name.clone().unwrap_or_default().to_string(),
        application_name: output.proplist.get_str("application.name"),
        binary: output.proplist.get_str("application.process.binary"),
        pid: output
            .proplist
            .get_str("application.process.id")
            .and_then(|pid_str| pid_str.parse::<u32>().ok()),
        owner_module: output.owner_module,
        client: output.client,
        device_index: output.source,
        volume: from_pulse(&output.volume),
        muted: output.mute,
        corked: output.corked,
        has_volume: output.has_volume,
        volume_writable: output.volume_writable,
        state: if output.corked {
            StreamState::Corked
        } else {
            StreamState::Running
        },
        sample_spec: convert_sample_spec(&output.sample_spec),
        channel_map: convert_channel_map(&output.channel_map),
        properties: collect_proplist(&output.proplist),
        media: extract_media_info(&output.proplist),
        buffer_latency: output.buffer_usec.0,
        device_latency: output.source_usec.0,
        resample_method: output.resample_method.as_ref().map(|s| s.to_string()),
        driver: output
            .driver
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default(),
        format: Some(format!("{:?}", output.format.get_encoding())),
    }
}
