use std::sync::Arc;

use derive_more::Debug;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, instrument};
use wayle_core::Property;
use wayle_traits::Reactive;
use zbus::Connection;

use super::core::{
    device::{
        input::{InputDeviceParams, LiveInputDeviceParams},
        output::{LiveOutputDeviceParams, OutputDeviceParams},
    },
    stream::{AudioStreamParams, LiveAudioStreamParams},
};
use crate::{
    backend::types::{CommandSender, EventSender},
    builder::AudioServiceBuilder,
    core::{
        device::{input::InputDevice, output::OutputDevice},
        stream::AudioStream,
    },
    error::Error,
    types::{device::DeviceKey, stream::StreamKey},
};

/// Pipewire Audio management service. See [crate-level docs](crate) for usage patterns.
#[derive(Debug)]
pub struct AudioService {
    #[debug(skip)]
    pub(crate) command_tx: CommandSender,
    #[debug(skip)]
    pub(crate) event_tx: EventSender,
    #[debug(skip)]
    pub(crate) cancellation_token: CancellationToken,
    #[debug(skip)]
    pub(crate) backend_handle: Option<JoinHandle<Result<(), Error>>>,
    #[debug(skip)]
    pub(crate) _connection: Option<Connection>,

    /// All PulseAudio sinks: speakers, headphones, Bluetooth outputs, virtual sinks.
    pub output_devices: Property<Vec<Arc<OutputDevice>>>,

    /// All PulseAudio sources: microphones, monitor sources, virtual inputs.
    pub input_devices: Property<Vec<Arc<InputDevice>>>,

    /// Current default sink, or `None` if unset.
    pub default_output: Property<Option<Arc<OutputDevice>>>,

    /// Current default source, or `None` if unset.
    pub default_input: Property<Option<Arc<InputDevice>>>,

    /// Applications currently playing audio.
    pub playback_streams: Property<Vec<Arc<AudioStream>>>,

    /// Applications currently recording audio.
    pub recording_streams: Property<Vec<Arc<AudioStream>>>,
}

impl AudioService {
    /// Creates a new audio service instance with default configuration.
    ///
    /// Initializes PulseAudio connection and discovers available devices and streams.
    ///
    /// # Errors
    /// Returns error if PulseAudio connection fails or service initialization fails.
    #[instrument]
    pub async fn new() -> Result<Arc<Self>, Error> {
        Self::builder().build().await
    }

    /// Creates a builder for configuring an AudioService.
    pub fn builder() -> AudioServiceBuilder {
        AudioServiceBuilder::new()
    }

    /// Returns a snapshot of the output device's current state.
    ///
    /// The returned [`OutputDevice`] properties will not update after this call.
    /// For live updates, see [`output_device_monitored`](Self::output_device_monitored).
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeviceNotFound`] if no sink exists with this key.
    #[instrument(skip(self), fields(device_key = ?key), err)]
    pub async fn output_device(&self, key: DeviceKey) -> Result<OutputDevice, Error> {
        OutputDevice::get(OutputDeviceParams {
            command_tx: &self.command_tx,
            device_key: key,
        })
        .await
    }

    /// Returns a live-updating output device instance.
    ///
    /// The returned [`OutputDevice`] properties update automatically when
    /// PulseAudio state changes. Monitoring stops when the `Arc` is dropped.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeviceNotFound`] if no sink exists with this key.
    #[instrument(skip(self), fields(device_key = ?key), err)]
    pub async fn output_device_monitored(
        &self,
        key: DeviceKey,
    ) -> Result<Arc<OutputDevice>, Error> {
        OutputDevice::get_live(LiveOutputDeviceParams {
            command_tx: &self.command_tx,
            event_tx: &self.event_tx,
            device_key: key,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Returns a snapshot of the input device's current state.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeviceNotFound`] if no source exists with this key.
    #[instrument(skip(self), fields(device_key = ?key), err)]
    pub async fn input_device(&self, key: DeviceKey) -> Result<InputDevice, Error> {
        InputDevice::get(InputDeviceParams {
            command_tx: &self.command_tx,
            device_key: key,
        })
        .await
    }

    /// Returns a live-updating input device instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::DeviceNotFound`] if no source exists with this key.
    #[instrument(skip(self), fields(device_key = ?key), err)]
    pub async fn input_device_monitored(&self, key: DeviceKey) -> Result<Arc<InputDevice>, Error> {
        InputDevice::get_live(LiveInputDeviceParams {
            command_tx: &self.command_tx,
            event_tx: &self.event_tx,
            device_key: key,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }

    /// Returns a snapshot of the audio stream's current state.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StreamNotFound`] if no stream exists with this key.
    #[instrument(skip(self), fields(stream_key = ?key), err)]
    pub async fn audio_stream(&self, key: StreamKey) -> Result<AudioStream, Error> {
        AudioStream::get(AudioStreamParams {
            command_tx: &self.command_tx,
            stream_key: key,
        })
        .await
    }

    /// Returns a live-updating audio stream instance.
    ///
    /// # Errors
    ///
    /// Returns [`Error::StreamNotFound`] if no stream exists with this key.
    #[instrument(skip(self), fields(stream_key = ?key), err)]
    pub async fn audio_stream_monitored(&self, key: StreamKey) -> Result<Arc<AudioStream>, Error> {
        AudioStream::get_live(LiveAudioStreamParams {
            command_tx: &self.command_tx,
            event_tx: &self.event_tx,
            stream_key: key,
            cancellation_token: &self.cancellation_token,
        })
        .await
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        self.cancellation_token.cancel();

        let Some(handle) = self.backend_handle.take() else {
            return;
        };

        let Ok(rt) = tokio::runtime::Handle::try_current() else {
            return;
        };

        let result = tokio::task::block_in_place(|| rt.block_on(handle));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => error!(error = %e, "PulseAudio backend shutdown error"),
            Err(e) => error!(error = %e, "PulseAudio backend task panicked"),
        }
    }
}
