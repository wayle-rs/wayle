use libpulse_binding::context::State as ContextState;

use super::types::{device::DeviceType, stream::StreamType};

/// Required component that was missing when attempting to start monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingMonitoringComponent {
    /// Cancellation token was not provided.
    CancellationToken,
    /// Event sender was not provided.
    EventSender,
}

impl std::fmt::Display for MissingMonitoringComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CancellationToken => write!(f, "cancellation token"),
            Self::EventSender => write!(f, "event sender"),
        }
    }
}

/// PulseAudio service errors.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Cannot create PulseAudio context.
    #[error("cannot create pulseaudio context")]
    ContextCreationFailed,

    /// Cannot connect to PulseAudio server.
    #[error("cannot connect to pulseaudio server")]
    ConnectionFailed(#[source] libpulse_binding::error::PAErr),

    /// PulseAudio context entered a failed state.
    #[error("pulseaudio context entered failed state: {0:?}")]
    ContextStateFailed(ContextState),

    /// Device not found.
    #[error("device {index:?} ({device_type:?}) not found")]
    DeviceNotFound {
        /// Device index that was not found.
        index: u32,
        /// Type of device (input/output).
        device_type: DeviceType,
    },

    /// Stream not found.
    #[error("stream {index:?} ({stream_type:?}) not found")]
    StreamNotFound {
        /// Stream index that was not found.
        index: u32,
        /// Type of stream.
        stream_type: StreamType,
    },

    /// Command channel disconnected.
    #[error("command channel disconnected")]
    CommandChannelDisconnected,

    /// Lock poisoned due to panic in another thread.
    #[error("shared data lock poisoned")]
    LockPoisoned,

    /// Monitoring cannot start because a required component was not provided.
    #[error("cannot start monitoring: {0} not available")]
    MonitoringNotInitialized(MissingMonitoringComponent),

    /// Cannot connect to D-Bus session bus.
    #[error("cannot connect to dbus session bus")]
    DbusConnectionFailed(#[source] zbus::Error),

    /// Cannot register D-Bus object.
    #[error("cannot register dbus object at {path}")]
    DbusObjectRegistrationFailed {
        /// The D-Bus path where registration was attempted.
        path: &'static str,
        /// The underlying zbus error.
        #[source]
        source: zbus::Error,
    },

    /// Cannot acquire D-Bus service name.
    #[error("cannot acquire dbus name {name}")]
    DbusNameAcquisitionFailed {
        /// The D-Bus name that could not be acquired.
        name: &'static str,
        /// The underlying zbus error.
        #[source]
        source: zbus::Error,
    },
}
