pub(crate) mod device;
pub(crate) mod server;
pub(crate) mod stream;

use tokio::sync::oneshot;

use crate::{
    error::Error,
    types::{
        device::{Device, DeviceKey},
        stream::{StreamInfo, StreamKey},
    },
    volume::types::Volume,
};

#[derive(Debug)]
pub(crate) enum Command {
    GetDevice {
        device_key: DeviceKey,
        responder: oneshot::Sender<Result<Device, Error>>,
    },
    GetStream {
        stream_key: StreamKey,
        responder: oneshot::Sender<Result<StreamInfo, Error>>,
    },
    SetVolume {
        device_key: DeviceKey,
        volume: Volume,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetMute {
        device_key: DeviceKey,
        muted: bool,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetStreamVolume {
        stream_key: StreamKey,
        volume: Volume,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetStreamMute {
        stream_key: StreamKey,
        muted: bool,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetDefaultInput {
        device_key: DeviceKey,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetDefaultOutput {
        device_key: DeviceKey,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    MoveStream {
        stream_key: StreamKey,
        device_key: DeviceKey,
        responder: oneshot::Sender<Result<(), Error>>,
    },
    SetPort {
        device_key: DeviceKey,
        port: String,
        responder: oneshot::Sender<Result<(), Error>>,
    },
}
