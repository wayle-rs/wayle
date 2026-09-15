use std::sync::Arc;

use libpulse_binding::{
    callbacks::ListResult,
    context::{Context, subscribe::Facility},
    volume::ChannelVolumes,
};

use crate::{
    backend::{
        conversion::stream::{from_sink_input, from_source_output},
        types::{EventSender, StreamStore},
    },
    events::AudioEvent,
    types::stream::{StreamInfo, StreamKey, StreamType},
};

pub(crate) fn trigger_discovery(context: &Context, streams: &StreamStore, events_tx: &EventSender) {
    let introspect = context.introspect();

    let streams_clone = Arc::clone(streams);
    let events_tx_clone = events_tx.clone();
    introspect.get_sink_input_info_list(move |sink_input_list| {
        if let ListResult::Item(sink_input) = sink_input_list {
            let stream_info = from_sink_input(sink_input);
            let stream_key = stream_info.key();
            process_stream_update(stream_key, stream_info, &streams_clone, &events_tx_clone);
        }
    });

    let streams_clone = Arc::clone(streams);
    let events_tx_clone = events_tx.clone();
    introspect.get_source_output_info_list(move |source_output_list| {
        if let ListResult::Item(source_output) = source_output_list {
            let stream_info = from_source_output(source_output);
            let stream_key = stream_info.key();
            process_stream_update(stream_key, stream_info, &streams_clone, &events_tx_clone);
        }
    });
}

pub(crate) fn trigger_refresh(
    context: &Context,
    streams: &StreamStore,
    events_tx: &EventSender,
    stream_key: StreamKey,
    facility: Facility,
) {
    let introspect = context.introspect();
    let streams_clone = Arc::clone(streams);
    let events_tx_clone = events_tx.clone();

    match facility {
        Facility::SinkInput => {
            introspect.get_sink_input_info(stream_key.index, move |input_list| {
                if let ListResult::Item(input) = input_list {
                    let stream_info = from_sink_input(input);
                    process_stream_update(
                        stream_key,
                        stream_info,
                        &streams_clone,
                        &events_tx_clone,
                    );
                }
            });
        }
        Facility::SourceOutput => {
            introspect.get_source_output_info(stream_key.index, move |output_list| {
                if let ListResult::Item(output) = output_list {
                    let stream_info = from_source_output(output);
                    process_stream_update(
                        stream_key,
                        stream_info,
                        &streams_clone,
                        &events_tx_clone,
                    );
                }
            });
        }
        _ => {}
    }
}

pub(crate) fn process_stream_update(
    stream_key: StreamKey,
    stream_data: StreamInfo,
    streams: &StreamStore,
    events_tx: &EventSender,
) {
    let Ok(mut streams_guard) = streams.write() else {
        return;
    };

    let is_new = !streams_guard.contains_key(&stream_key);
    streams_guard.insert(stream_key, stream_data.clone());

    let event = if is_new {
        AudioEvent::StreamAdded(stream_data)
    } else {
        AudioEvent::StreamChanged(stream_data)
    };

    let _ = events_tx.send(event);
}

pub(crate) fn set_stream_volume(
    context: &Context,
    stream_key: StreamKey,
    volume: ChannelVolumes,
    streams: &StreamStore,
) {
    let streams_clone = Arc::clone(streams);
    let mut introspect = context.introspect();

    let stream_info = {
        if let Ok(streams_guard) = streams_clone.read() {
            streams_guard.get(&stream_key).cloned()
        } else {
            return;
        }
    };

    if let Some(info) = stream_info {
        match info.stream_type {
            StreamType::Playback => {
                introspect.set_sink_input_volume(stream_key.index, &volume, None);
            }
            StreamType::Record => {
                introspect.set_source_output_volume(stream_key.index, &volume, None);
            }
        }
    }
}

pub(crate) fn set_stream_mute(
    context: &Context,
    stream_key: StreamKey,
    muted: bool,
    streams: &StreamStore,
) {
    let streams_clone = Arc::clone(streams);
    let mut introspect = context.introspect();

    let stream_info = {
        if let Ok(streams_guard) = streams_clone.read() {
            streams_guard.get(&stream_key).cloned()
        } else {
            return;
        }
    };

    if let Some(info) = stream_info {
        match info.stream_type {
            StreamType::Playback => {
                introspect.set_sink_input_mute(stream_key.index, muted, None);
            }
            StreamType::Record => {
                introspect.set_source_output_mute(stream_key.index, muted, None);
            }
        }
    }
}

pub(crate) fn move_stream(
    context: &Context,
    stream_key: StreamKey,
    device_key: crate::types::device::DeviceKey,
    streams: &StreamStore,
) {
    let streams_clone = Arc::clone(streams);
    let mut introspect = context.introspect();

    let stream_info = {
        if let Ok(streams_guard) = streams_clone.read() {
            streams_guard.get(&stream_key).cloned()
        } else {
            return;
        }
    };

    if let Some(info) = stream_info {
        match info.stream_type {
            StreamType::Playback => {
                introspect.move_sink_input_by_index(stream_key.index, device_key.index, None);
            }
            StreamType::Record => {
                introspect.move_source_output_by_index(stream_key.index, device_key.index, None);
            }
        }
    }
}
