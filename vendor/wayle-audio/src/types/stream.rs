use std::collections::HashMap;

use super::format::{ChannelMap, SampleSpec};
use crate::volume::types::Volume;

/// Stream state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamState {
    /// Stream is creating.
    Creating,
    /// Stream is ready.
    Ready,
    /// Stream is actively playing or recording.
    Running,
    /// Stream is paused (corked in PulseAudio).
    Corked,
    /// Stream has failed.
    Failed,
    /// Stream has been terminated.
    Terminated,
    /// Stream has been unlinked.
    Unlinked,
}

/// Stream type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StreamType {
    /// Playback stream (sink input)
    Playback,
    /// Recording stream (source output)
    Record,
}

/// Stream key for unique identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StreamKey {
    /// Stream index
    pub index: u32,
    /// Stream type
    pub stream_type: StreamType,
}

impl StreamKey {
    /// Create a new stream key
    pub fn new(index: u32, stream_type: StreamType) -> Self {
        Self { index, stream_type }
    }
}

/// Media information for a stream
#[derive(Debug, Clone, PartialEq)]
pub struct MediaInfo {
    /// Track title
    pub title: Option<String>,
    /// Artist name
    pub artist: Option<String>,
    /// Album name
    pub album: Option<String>,
    /// Icon name
    pub icon_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StreamInfo {
    pub index: u32,
    pub stream_type: StreamType,
    pub name: String,
    pub application_name: Option<String>,
    pub binary: Option<String>,
    pub pid: Option<u32>,
    pub owner_module: Option<u32>,
    pub client: Option<u32>,
    pub device_index: u32,
    pub volume: Volume,
    pub muted: bool,
    pub corked: bool,
    pub has_volume: bool,
    pub volume_writable: bool,
    pub state: StreamState,
    pub sample_spec: SampleSpec,
    pub channel_map: ChannelMap,
    pub properties: HashMap<String, String>,
    pub media: MediaInfo,
    pub buffer_latency: u64,
    pub device_latency: u64,
    pub resample_method: Option<String>,
    pub driver: String,
    pub format: Option<String>,
}

impl StreamInfo {
    pub(crate) fn key(&self) -> StreamKey {
        StreamKey {
            index: self.index,
            stream_type: self.stream_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stream_info(
        stream_type: StreamType,
        properties: HashMap<String, String>,
    ) -> StreamInfo {
        use crate::{
            types::format::{ChannelMap, SampleFormat, SampleSpec},
            volume::types::Volume,
        };

        StreamInfo {
            index: 0,
            stream_type,
            name: String::from("test"),
            application_name: None,
            binary: None,
            pid: None,
            owner_module: None,
            client: None,
            device_index: 0,
            volume: Volume::mono(1.0),
            muted: false,
            corked: false,
            has_volume: true,
            volume_writable: true,
            state: StreamState::Running,
            sample_spec: SampleSpec {
                format: SampleFormat::S16LE,
                rate: 44100,
                channels: 2,
            },
            channel_map: ChannelMap {
                channels: 2,
                positions: vec![],
            },
            properties,
            media: MediaInfo {
                title: None,
                artist: None,
                album: None,
                icon_name: None,
            },
            buffer_latency: 0,
            device_latency: 0,
            resample_method: None,
            driver: String::from("test-driver"),
            format: None,
        }
    }

    #[test]
    fn key_returns_correct_stream_key_for_record_stream() {
        let mut info = create_test_stream_info(StreamType::Record, HashMap::new());
        info.index = 42;

        let key = info.key();

        assert_eq!(key.index, 42);
        assert_eq!(key.stream_type, StreamType::Record);
    }

    #[test]
    fn key_returns_correct_stream_key_for_playback_stream() {
        let mut info = create_test_stream_info(StreamType::Playback, HashMap::new());
        info.index = 123;

        let key = info.key();

        assert_eq!(key.index, 123);
        assert_eq!(key.stream_type, StreamType::Playback);
    }
}
