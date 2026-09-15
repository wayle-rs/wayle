pub(crate) mod controls;
pub(crate) mod monitoring;
pub(crate) mod types;

use std::{collections::HashMap, sync::Arc};

use controls::InputDeviceController;
use derive_more::Debug;
use libpulse_binding::time::MicroSeconds;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
pub(crate) use types::{InputDeviceParams, LiveInputDeviceParams};
use wayle_core::Property;
use wayle_traits::{ModelMonitoring, Reactive};

use crate::{
    backend::{
        commands::Command,
        types::{CommandSender, EventSender},
    },
    error::Error,
    types::{
        device::{Device, DeviceKey, DevicePort, DeviceState, DeviceType, SourceInfo},
        format::{AudioFormat, ChannelMap, SampleSpec},
    },
    volume::types::Volume,
};

/// PulseAudio source with reactive properties and control methods.
///
/// Instances from [`AudioService`] fields are live (properties auto-update).
/// Instances from [`AudioService::input_device`] are snapshots (frozen state).
///
/// # Control Methods
///
/// - [`set_volume`](Self::set_volume) - Adjust input gain
/// - [`set_mute`](Self::set_mute) - Mute or unmute
/// - [`set_port`](Self::set_port) - Switch input port
/// - [`set_as_default`](Self::set_as_default) - Make this the default input
///
/// [`AudioService`]: crate::AudioService
/// [`AudioService::input_device`]: crate::AudioService::input_device
#[derive(Clone, Debug)]
pub struct InputDevice {
    /// Command sender for backend operations
    #[debug(skip)]
    command_tx: CommandSender,

    /// Event sender for monitoring (only for live instances)
    #[debug(skip)]
    event_tx: Option<EventSender>,

    /// Cancellation token for monitoring (only for live instances)
    #[debug(skip)]
    cancellation_token: Option<CancellationToken>,

    /// Device key for identification
    pub key: DeviceKey,

    /// Device name (internal identifier)
    pub name: Property<String>,

    /// Human-readable description
    pub description: Property<String>,

    /// Card index this device belongs to
    pub card_index: Property<Option<u32>>,

    /// Index of the owning module
    pub owner_module: Property<Option<u32>>,

    /// Driver name
    pub driver: Property<String>,

    /// Device state
    pub state: Property<DeviceState>,

    /// Current volume levels
    pub volume: Property<Volume>,

    /// Base volume (reference level)
    pub base_volume: Property<Volume>,

    /// Number of volume steps for devices which do not support arbitrary volumes
    pub n_volume_steps: Property<u32>,

    /// Whether device is muted
    pub muted: Property<bool>,

    /// Device properties from PulseAudio
    pub properties: Property<HashMap<String, String>>,

    /// Available ports
    pub ports: Property<Vec<DevicePort>>,

    /// Currently active port
    pub active_port: Property<Option<String>>,

    /// Supported audio formats
    pub formats: Property<Vec<AudioFormat>>,

    /// Sample specification
    pub sample_spec: Property<SampleSpec>,

    /// Channel map
    pub channel_map: Property<ChannelMap>,

    /// Index of the sink being monitored (if this is a monitor source)
    pub monitor_of_sink: Property<Option<u32>>,

    /// Name of the sink being monitored (if this is a monitor source)
    pub monitor_of_sink_name: Property<Option<String>>,

    /// Whether this is a monitor source
    pub is_monitor: Property<bool>,

    /// Latency in microseconds
    pub latency: Property<MicroSeconds>,

    /// Configured latency in microseconds
    pub configured_latency: Property<MicroSeconds>,

    /// Device flags (raw flags from PulseAudio)
    pub flags: Property<u32>,
}

impl PartialEq for InputDevice {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Reactive for InputDevice {
    type Context<'a> = InputDeviceParams<'a>;
    type LiveContext<'a> = LiveInputDeviceParams<'a>;
    type Error = Error;

    async fn get(params: Self::Context<'_>) -> Result<Self, Self::Error> {
        let (tx, rx) = oneshot::channel();
        params
            .command_tx
            .send(Command::GetDevice {
                device_key: params.device_key,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        let device = rx.await.map_err(|_| Error::CommandChannelDisconnected)??;

        match device {
            Device::Source(source) => Ok(Self::from_source(
                &source,
                params.command_tx.clone(),
                None,
                None,
            )),
            Device::Sink(_) => Err(Error::DeviceNotFound {
                index: params.device_key.index,
                device_type: DeviceType::Input,
            }),
        }
    }

    async fn get_live(params: Self::LiveContext<'_>) -> Result<Arc<Self>, Self::Error> {
        let (tx, rx) = oneshot::channel();
        params
            .command_tx
            .send(Command::GetDevice {
                device_key: params.device_key,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        let device = rx.await.map_err(|_| Error::CommandChannelDisconnected)??;

        let device = match device {
            Device::Source(source) => Arc::new(Self::from_source(
                &source,
                params.command_tx.clone(),
                Some(params.event_tx.clone()),
                Some(params.cancellation_token.child_token()),
            )),
            Device::Sink(_) => {
                return Err(Error::DeviceNotFound {
                    index: params.device_key.index,
                    device_type: DeviceType::Input,
                });
            }
        };

        device.clone().start_monitoring().await?;

        Ok(device)
    }
}

impl InputDevice {
    pub(crate) fn from_source(
        source: &SourceInfo,
        command_tx: CommandSender,
        event_tx: Option<EventSender>,
        cancellation_token: Option<CancellationToken>,
    ) -> Self {
        Self {
            command_tx,
            event_tx,
            cancellation_token,
            key: source.key(),
            name: Property::new(source.device.name.clone()),
            description: Property::new(source.device.description.clone()),
            card_index: Property::new(source.device.card_index),
            owner_module: Property::new(source.device.owner_module),
            driver: Property::new(source.device.driver.clone()),
            state: Property::new(source.device.state),
            volume: Property::new(source.device.volume.clone()),
            base_volume: Property::new(source.device.base_volume.clone()),
            n_volume_steps: Property::new(source.device.n_volume_steps),
            muted: Property::new(source.device.muted),
            properties: Property::new(source.device.properties.clone()),
            ports: Property::new(source.device.ports.clone()),
            active_port: Property::new(source.device.active_port.clone()),
            formats: Property::new(source.device.formats.clone()),
            sample_spec: Property::new(source.device.sample_spec.clone()),
            channel_map: Property::new(source.device.channel_map.clone()),
            monitor_of_sink: Property::new(source.monitor_of_sink),
            monitor_of_sink_name: Property::new(source.monitor_of_sink_name.clone()),
            is_monitor: Property::new(source.is_monitor),
            latency: Property::new(source.device.latency),
            configured_latency: Property::new(source.device.configured_latency),
            flags: Property::new(source.device.flags),
        }
    }

    pub(crate) fn update_from_source(&self, source: &SourceInfo) {
        self.name.set(source.device.name.clone());
        self.description.set(source.device.description.clone());
        self.card_index.set(source.device.card_index);
        self.owner_module.set(source.device.owner_module);
        self.driver.set(source.device.driver.clone());
        self.state.set(source.device.state);
        self.volume.set(source.device.volume.clone());
        self.base_volume.set(source.device.base_volume.clone());
        self.n_volume_steps.set(source.device.n_volume_steps);
        self.muted.set(source.device.muted);
        self.properties.set(source.device.properties.clone());
        self.ports.set(source.device.ports.clone());
        self.active_port.set(source.device.active_port.clone());
        self.formats.set(source.device.formats.clone());
        self.sample_spec.set(source.device.sample_spec.clone());
        self.channel_map.set(source.device.channel_map.clone());
        self.monitor_of_sink.set(source.monitor_of_sink);
        self.monitor_of_sink_name
            .set(source.monitor_of_sink_name.clone());
        self.is_monitor.set(source.is_monitor);
        self.latency.set(source.device.latency);
        self.configured_latency
            .set(source.device.configured_latency);
        self.flags.set(source.device.flags);
    }

    /// Set the volume for this input device.
    ///
    /// # Errors
    /// Returns error if backend communication fails or device operation fails.
    pub async fn set_volume(&self, volume: Volume) -> Result<(), Error> {
        InputDeviceController::set_volume(&self.command_tx, self.key, volume).await
    }

    /// Set the mute state for this input device.
    ///
    /// # Errors
    /// Returns error if backend communication fails or device operation fails.
    pub async fn set_mute(&self, muted: bool) -> Result<(), Error> {
        InputDeviceController::set_mute(&self.command_tx, self.key, muted).await
    }

    /// Set the active port for this input device.
    ///
    /// # Errors
    /// Returns error if backend communication fails or device operation fails.
    pub async fn set_port(&self, port: String) -> Result<(), Error> {
        InputDeviceController::set_port(&self.command_tx, self.key, port).await
    }

    /// Set this device as the default input.
    ///
    /// # Errors
    /// Returns error if backend communication fails or device operation fails.
    pub async fn set_as_default(&self) -> Result<(), Error> {
        InputDeviceController::set_as_default(&self.command_tx, self.key).await
    }
}
