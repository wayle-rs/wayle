//! Shell IPC service exposing `com.wayle.Shell1` on the session bus.
//!
//! Provides bar visibility control (hide/show/toggle per monitor) and manual
//! OSD display via D-Bus methods, plus reactive [`ShellIpcState`] that shell
//! components watch to apply those commands.

mod bar;
mod dbus;
mod error;
mod osd;
mod state;

use std::sync::Arc;

pub use error::Error;
pub use state::{OsdDevice, OsdRequest, ShellIpcState};
use tracing::info;
use wayle_audio::AudioService;
use wayle_brightness::BrightnessService;
use wayle_config::ConfigService;
use wayle_ipc::shell_ipc::{SERVICE_NAME, SERVICE_PATH};
use zbus::Connection;

use self::dbus::ShellIpcDaemon;

/// Registers the `com.wayle.Shell1` D-Bus interface and holds the
/// [`ShellIpcState`] that shell components watch for visibility and OSD
/// commands.
pub struct ShellIpcService {
    state: ShellIpcState,
    _connection: Connection,
}

impl ShellIpcService {
    /// Connects to the session bus and registers the `com.wayle.Shell1` interface.
    ///
    /// The audio and brightness services are used to resolve the device named
    /// in an OSD request; when either is unavailable the matching OSD commands
    /// report that instead of failing silently.
    ///
    /// # Errors
    ///
    /// Returns an error if the session bus is unreachable or the D-Bus name
    /// is already claimed.
    pub async fn new(
        config: Arc<ConfigService>,
        audio: Option<Arc<AudioService>>,
        brightness: Option<Arc<BrightnessService>>,
    ) -> Result<Self, Error> {
        let state = ShellIpcState::new();

        let connection = Connection::session()
            .await
            .map_err(|err| Error::Connection(err.to_string()))?;

        let daemon = ShellIpcDaemon::new(state.clone(), config, audio, brightness);

        connection
            .object_server()
            .at(SERVICE_PATH, daemon)
            .await
            .map_err(|err| Error::Registration(err.to_string()))?;

        connection
            .request_name(SERVICE_NAME)
            .await
            .map_err(|err| Error::NameRequest(err.to_string()))?;

        info!("Shell IPC service registered at {SERVICE_NAME}");

        Ok(Self {
            state,
            _connection: connection,
        })
    }

    /// Reactive state that shell components subscribe to for IPC commands.
    pub fn state(&self) -> ShellIpcState {
        self.state.clone()
    }
}
