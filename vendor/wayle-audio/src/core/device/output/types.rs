use tokio_util::sync::CancellationToken;

use crate::{
    backend::types::{CommandSender, EventSender},
    types::device::DeviceKey,
};

#[doc(hidden)]
#[allow(private_interfaces)]
pub struct OutputDeviceParams<'a> {
    pub command_tx: &'a CommandSender,
    pub device_key: DeviceKey,
}

#[doc(hidden)]
#[allow(private_interfaces)]
pub struct LiveOutputDeviceParams<'a> {
    pub command_tx: &'a CommandSender,
    pub event_tx: &'a EventSender,
    pub device_key: DeviceKey,
    pub cancellation_token: &'a CancellationToken,
}
