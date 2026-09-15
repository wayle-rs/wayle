//! Builder for configuring an AudioService.

use std::sync::Arc;

use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use tracing::info;
use wayle_core::Property;
use wayle_traits::ServiceMonitoring;
use zbus::Connection;

use crate::{
    backend::PulseBackend,
    dbus::{AudioDaemon, SERVICE_NAME, SERVICE_PATH},
    error::Error,
    service::AudioService,
};

/// Builder for configuring and creating an AudioService instance.
///
/// Allows optional D-Bus daemon registration for external control.
#[derive(Default)]
pub struct AudioServiceBuilder {
    register_daemon: bool,
}

impl AudioServiceBuilder {
    /// Creates a new AudioServiceBuilder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables D-Bus daemon registration for external control.
    ///
    /// When enabled, the service will register itself on the session bus
    /// at `com.wayle.Audio1`, allowing CLI tools and other applications
    /// to control audio devices.
    pub fn with_daemon(mut self) -> Self {
        self.register_daemon = true;
        self
    }

    /// Builds and initializes the AudioService.
    ///
    /// This will establish a PulseAudio connection and start monitoring
    /// for device changes. If `with_daemon()` was called, the service
    /// will also register on the session bus for external control.
    ///
    /// # Errors
    /// Returns error if PulseAudio connection fails or monitoring cannot be started.
    pub async fn build(self) -> Result<Arc<AudioService>, Error> {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, _) = broadcast::channel(100);
        let cancellation_token = CancellationToken::new();

        let output_devices = Property::new(Vec::new());
        let input_devices = Property::new(Vec::new());
        let default_output = Property::new(None);
        let default_input = Property::new(None);
        let playback_streams = Property::new(Vec::new());
        let recording_streams = Property::new(Vec::new());

        let backend_handle = PulseBackend::start(
            command_rx,
            event_tx.clone(),
            cancellation_token.child_token(),
        );

        let connection = if self.register_daemon {
            let conn = Connection::session()
                .await
                .map_err(Error::DbusConnectionFailed)?;
            Some(conn)
        } else {
            None
        };

        let service = Arc::new(AudioService {
            command_tx,
            event_tx,
            cancellation_token,
            backend_handle: Some(backend_handle),
            _connection: connection.clone(),
            output_devices,
            input_devices,
            default_output,
            default_input,
            playback_streams,
            recording_streams,
        });

        service.start_monitoring().await?;

        if let Some(connection) = connection {
            let daemon = AudioDaemon {
                service: Arc::clone(&service),
            };

            connection
                .object_server()
                .at(SERVICE_PATH, daemon)
                .await
                .map_err(|source| Error::DbusObjectRegistrationFailed {
                    path: SERVICE_PATH,
                    source,
                })?;

            connection
                .request_name(SERVICE_NAME)
                .await
                .map_err(|source| Error::DbusNameAcquisitionFailed {
                    name: SERVICE_NAME,
                    source,
                })?;

            info!("Audio service registered at {SERVICE_NAME}");
        }

        Ok(service)
    }
}
