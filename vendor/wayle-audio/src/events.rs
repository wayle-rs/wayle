use super::types::{
    device::{Device, DeviceKey},
    stream::{StreamInfo, StreamKey},
};

#[derive(Debug, Clone)]
pub(crate) enum AudioEvent {
    DeviceAdded(Device),
    DeviceChanged(Device),
    DeviceRemoved(DeviceKey),
    StreamAdded(StreamInfo),
    StreamChanged(StreamInfo),
    StreamRemoved(StreamKey),
    DefaultInputChanged(Option<Device>),
    DefaultOutputChanged(Option<Device>),
}
