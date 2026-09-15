pub(crate) mod controls;
pub(crate) mod monitoring;
pub(crate) mod types;

use std::{collections::HashMap, sync::Arc};

use controls::AudioStreamController;
use derive_more::Debug;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
pub(crate) use types::{AudioStreamParams, LiveAudioStreamParams};
use wayle_core::Property;
use wayle_traits::{ModelMonitoring, Reactive};

use crate::{
    backend::{
        commands::Command,
        types::{CommandSender, EventSender},
    },
    error::Error,
    types::{
        device::DeviceKey,
        format::{ChannelMap, SampleSpec},
        stream::{MediaInfo, StreamInfo, StreamKey, StreamState},
    },
    volume::types::Volume,
};

/// PulseAudio stream (playback or recording) with reactive properties.
///
/// Instances from [`AudioService`] fields are live (properties auto-update).
/// Instances from [`AudioService::audio_stream`] are snapshots (frozen state).
///
/// # Control Methods
///
/// - [`set_volume`](Self::set_volume) - Adjust stream volume
/// - [`set_mute`](Self::set_mute) - Mute or unmute
/// - [`move_to_device`](Self::move_to_device) - Route to a different device
///
/// [`AudioService`]: crate::AudioService
/// [`AudioService::audio_stream`]: crate::AudioService::audio_stream
#[derive(Clone, Debug)]
pub struct AudioStream {
    /// Command sender for backend operations
    #[debug(skip)]
    command_tx: CommandSender,

    /// Event sender for monitoring (only for live instances)
    #[debug(skip)]
    event_tx: Option<EventSender>,

    /// Cancellation token for monitoring (only for live instances)
    #[debug(skip)]
    pub(crate) cancellation_token: Option<CancellationToken>,

    /// Stream key for identification
    pub key: StreamKey,

    /// Stream name
    pub name: Property<String>,

    /// Application name
    pub application_name: Property<Option<String>>,

    /// Application binary path
    pub binary: Property<Option<String>>,

    /// Process ID
    pub pid: Property<Option<u32>>,

    /// Index of the owning module
    pub owner_module: Property<Option<u32>>,

    /// Index of the client this stream belongs to
    pub client: Property<Option<u32>>,

    /// Stream state
    pub state: Property<StreamState>,

    /// Current volume levels
    pub volume: Property<Volume>,

    /// Whether stream is muted
    pub muted: Property<bool>,

    /// Whether stream is corked (paused)
    pub corked: Property<bool>,

    /// Whether stream has volume control
    pub has_volume: Property<bool>,

    /// Whether volume is writable by clients
    pub volume_writable: Property<bool>,

    /// Device index this stream is connected to
    pub device_index: Property<u32>,

    /// Sample specification
    pub sample_spec: Property<SampleSpec>,

    /// Channel map
    pub channel_map: Property<ChannelMap>,

    /// Stream properties from PulseAudio
    pub properties: Property<HashMap<String, String>>,

    /// Media information
    pub media: Property<MediaInfo>,

    /// Buffer latency in microseconds
    pub buffer_latency: Property<u64>,

    /// Device latency in microseconds
    pub device_latency: Property<u64>,

    /// Resample method
    pub resample_method: Property<Option<String>>,

    /// Driver name
    pub driver: Property<String>,

    /// Format information for the stream
    pub format: Property<Option<String>>,
}

impl PartialEq for AudioStream {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Reactive for AudioStream {
    type Context<'a> = AudioStreamParams<'a>;
    type LiveContext<'a> = LiveAudioStreamParams<'a>;
    type Error = Error;

    async fn get(params: Self::Context<'_>) -> Result<Self, Self::Error> {
        let (tx, rx) = oneshot::channel();
        params
            .command_tx
            .send(Command::GetStream {
                stream_key: params.stream_key,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        let stream_info = rx.await.map_err(|_| Error::CommandChannelDisconnected)??;
        Ok(Self::from_info(
            stream_info,
            params.command_tx.clone(),
            None,
            None,
        ))
    }

    async fn get_live(params: Self::LiveContext<'_>) -> Result<Arc<Self>, Self::Error> {
        let (tx, rx) = oneshot::channel();
        params
            .command_tx
            .send(Command::GetStream {
                stream_key: params.stream_key,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        let stream_info = rx.await.map_err(|_| Error::CommandChannelDisconnected)??;
        let stream = Arc::new(Self::from_info(
            stream_info,
            params.command_tx.clone(),
            Some(params.event_tx.clone()),
            Some(params.cancellation_token.child_token()),
        ));

        stream.clone().start_monitoring().await?;

        Ok(stream)
    }
}

impl AudioStream {
    /// Create stream from info snapshot
    pub(crate) fn from_info(
        info: StreamInfo,
        command_tx: CommandSender,
        event_tx: Option<EventSender>,
        cancellation_token: Option<CancellationToken>,
    ) -> Self {
        Self {
            command_tx,
            event_tx,
            cancellation_token,
            key: info.key(),
            name: Property::new(info.name),
            application_name: Property::new(info.application_name),
            binary: Property::new(info.binary),
            pid: Property::new(info.pid),
            owner_module: Property::new(info.owner_module),
            client: Property::new(info.client),
            state: Property::new(info.state),
            volume: Property::new(info.volume),
            muted: Property::new(info.muted),
            corked: Property::new(info.corked),
            has_volume: Property::new(info.has_volume),
            volume_writable: Property::new(info.volume_writable),
            device_index: Property::new(info.device_index),
            sample_spec: Property::new(info.sample_spec),
            channel_map: Property::new(info.channel_map),
            properties: Property::new(info.properties),
            media: Property::new(info.media),
            buffer_latency: Property::new(info.buffer_latency),
            device_latency: Property::new(info.device_latency),
            resample_method: Property::new(info.resample_method),
            driver: Property::new(info.driver),
            format: Property::new(info.format),
        }
    }

    /// Update stream properties from new info
    pub(crate) fn update_from_info(&self, info: &StreamInfo) {
        self.name.set(info.name.clone());
        self.application_name.set(info.application_name.clone());
        self.binary.set(info.binary.clone());
        self.pid.set(info.pid);
        self.owner_module.set(info.owner_module);
        self.client.set(info.client);
        self.state.set(info.state);
        self.volume.set(info.volume.clone());
        self.muted.set(info.muted);
        self.corked.set(info.corked);
        self.has_volume.set(info.has_volume);
        self.volume_writable.set(info.volume_writable);
        self.device_index.set(info.device_index);
        self.sample_spec.set(info.sample_spec.clone());
        self.channel_map.set(info.channel_map.clone());
        self.properties.set(info.properties.clone());
        self.media.set(info.media.clone());
        self.buffer_latency.set(info.buffer_latency);
        self.device_latency.set(info.device_latency);
        self.resample_method.set(info.resample_method.clone());
        self.driver.set(info.driver.clone());
        self.format.set(info.format.clone());
    }

    /// Set the volume for this audio stream.
    ///
    /// # Errors
    /// Returns error if backend communication fails or stream operation fails.
    pub async fn set_volume(&self, volume: Volume) -> Result<(), Error> {
        AudioStreamController::set_volume(&self.command_tx, self.key, volume).await
    }

    /// Set the mute state for this audio stream.
    ///
    /// # Errors
    /// Returns error if backend communication fails or stream operation fails.
    pub async fn set_mute(&self, muted: bool) -> Result<(), Error> {
        AudioStreamController::set_mute(&self.command_tx, self.key, muted).await
    }

    /// Move this stream to a different device.
    ///
    /// # Errors
    /// Returns error if backend communication fails or device doesn't exist.
    pub async fn move_to_device(&self, device_key: DeviceKey) -> Result<(), Error> {
        AudioStreamController::move_to_device(&self.command_tx, self.key, device_key).await
    }
}
