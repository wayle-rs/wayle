use std::sync::Arc;

use tracing::debug;
use wayle_traits::ModelMonitoring;

use crate::{
    core::stream::AudioStream,
    error::{Error, MissingMonitoringComponent},
    events::AudioEvent,
    types::stream::StreamState,
};

impl ModelMonitoring for AudioStream {
    type Error = Error;

    async fn start_monitoring(self: Arc<Self>) -> Result<(), Self::Error> {
        let Some(ref cancellation_token) = self.cancellation_token else {
            return Err(Error::MonitoringNotInitialized(
                MissingMonitoringComponent::CancellationToken,
            ));
        };

        let Some(ref event_tx) = self.event_tx else {
            return Err(Error::MonitoringNotInitialized(
                MissingMonitoringComponent::EventSender,
            ));
        };

        let weak_stream = Arc::downgrade(&self);
        let stream_key = self.key;
        let cancellation_token = cancellation_token.clone();
        let mut event_rx = event_tx.subscribe();

        tokio::spawn(async move {
            loop {
                let Some(stream) = weak_stream.upgrade() else {
                    return;
                };

                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        debug!("AudioStream monitor cancelled for {:?}", stream_key);
                        return;
                    }
                    Ok(event) = event_rx.recv() => {
                        match event {
                            AudioEvent::StreamChanged(info) if info.key() == stream_key => {
                                stream.update_from_info(&info);
                            }
                            AudioEvent::StreamRemoved(key) if key == stream_key => {
                                stream.state.set(StreamState::Terminated);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
