use tokio_util::sync::CancellationToken;

use crate::{
    backend::types::{CommandSender, EventSender},
    types::stream::StreamKey,
};

#[doc(hidden)]
#[allow(private_interfaces)]
pub struct AudioStreamParams<'a> {
    pub command_tx: &'a CommandSender,
    pub stream_key: StreamKey,
}

#[doc(hidden)]
#[allow(private_interfaces)]
pub struct LiveAudioStreamParams<'a> {
    pub command_tx: &'a CommandSender,
    pub event_tx: &'a EventSender,
    pub stream_key: StreamKey,
    pub cancellation_token: &'a CancellationToken,
}
