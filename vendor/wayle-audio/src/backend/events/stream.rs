use libpulse_binding::context::subscribe::{Facility, Operation};

use crate::{
    backend::types::{EventSender, InternalCommandSender, InternalRefresh, StreamStore},
    events::AudioEvent,
    types::stream::{StreamKey, StreamType},
};

pub(crate) async fn handle_change(
    facility: Facility,
    operation: Operation,
    stream_index: u32,
    streams: &StreamStore,
    events_tx: &EventSender,
    command_tx: &InternalCommandSender,
) {
    let stream_type = match facility {
        Facility::SinkInput => StreamType::Playback,
        Facility::SourceOutput => StreamType::Record,
        _ => return,
    };

    let stream_key = StreamKey {
        stream_type,
        index: stream_index,
    };

    match operation {
        Operation::Removed => {
            let removed_stream = if let Ok(mut streams_guard) = streams.write() {
                streams_guard.remove(&stream_key)
            } else {
                None
            };

            if removed_stream.is_some() {
                let _ = events_tx.send(AudioEvent::StreamRemoved(stream_key));
            }
        }
        Operation::New | Operation::Changed => {
            let _ = command_tx.send(InternalRefresh::Stream {
                stream_key,
                facility,
            });
        }
    }
}
