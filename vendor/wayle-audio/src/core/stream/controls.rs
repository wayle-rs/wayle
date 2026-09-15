use tokio::sync::oneshot;

use crate::{
    backend::{commands::Command, types::CommandSender},
    error::Error,
    types::{device::DeviceKey, stream::StreamKey},
    volume::types::Volume,
};

pub(crate) struct AudioStreamController;

impl AudioStreamController {
    pub async fn set_volume(
        command_tx: &CommandSender,
        stream_key: StreamKey,
        volume: Volume,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();

        command_tx
            .send(Command::SetStreamVolume {
                stream_key,
                volume,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        rx.await.map_err(|_| Error::CommandChannelDisconnected)?
    }

    pub async fn set_mute(
        command_tx: &CommandSender,
        stream_key: StreamKey,
        muted: bool,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();

        command_tx
            .send(Command::SetStreamMute {
                stream_key,
                muted,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        rx.await.map_err(|_| Error::CommandChannelDisconnected)?
    }

    pub async fn move_to_device(
        command_tx: &CommandSender,
        stream_key: StreamKey,
        device_key: DeviceKey,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();

        command_tx
            .send(Command::MoveStream {
                stream_key,
                device_key,
                responder: tx,
            })
            .map_err(|_| Error::CommandChannelDisconnected)?;

        rx.await.map_err(|_| Error::CommandChannelDisconnected)?
    }
}
