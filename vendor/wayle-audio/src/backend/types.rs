use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use libpulse_binding::{
    context::subscribe::{Facility, Operation},
    volume::ChannelVolumes,
};
use tokio::sync::{broadcast, mpsc};

use super::commands::Command;
use crate::{
    events::AudioEvent,
    types::{
        device::{Device, DeviceKey},
        stream::{StreamInfo, StreamKey},
    },
};

pub(crate) type DeviceStore = Arc<RwLock<HashMap<DeviceKey, Device>>>;

pub(crate) type StreamStore = Arc<RwLock<HashMap<StreamKey, StreamInfo>>>;

pub(crate) type DefaultDevice = Arc<RwLock<Option<Device>>>;

pub(crate) type EventSender = broadcast::Sender<AudioEvent>;

pub(crate) type CommandSender = mpsc::UnboundedSender<Command>;

pub(crate) type CommandReceiver = mpsc::UnboundedReceiver<Command>;

pub(super) type InternalCommandSender = mpsc::UnboundedSender<InternalRefresh>;

#[derive(Debug, Clone)]
pub(crate) enum ChangeNotification {
    Device {
        facility: Facility,
        operation: Operation,
        index: u32,
    },
    Stream {
        facility: Facility,
        operation: Operation,
        index: u32,
    },
    Server {
        operation: Operation,
    },
}

#[derive(Debug)]
pub(crate) enum InternalRefresh {
    Devices,
    Streams,
    ServerInfo,
    Device {
        device_key: DeviceKey,
        facility: Facility,
    },
    Stream {
        stream_key: StreamKey,
        facility: Facility,
    },
}

#[derive(Debug)]
pub(crate) enum ExternalCommand {
    SetDeviceVolume {
        device_key: DeviceKey,
        volume: ChannelVolumes,
    },
    SetDeviceMute {
        device_key: DeviceKey,
        muted: bool,
    },
    SetStreamVolume {
        stream_key: StreamKey,
        volume: ChannelVolumes,
    },
    SetStreamMute {
        stream_key: StreamKey,
        muted: bool,
    },
    SetDefaultInput {
        device_key: DeviceKey,
    },
    SetDefaultOutput {
        device_key: DeviceKey,
    },
    MoveStream {
        stream_key: StreamKey,
        device_key: DeviceKey,
    },
    SetPort {
        device_key: DeviceKey,
        port: String,
    },
}
