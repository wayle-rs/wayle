//! Error types for the Triad service.

use std::{
    fmt::{self, Display, Formatter},
    io,
};

/// Which socket role a connection error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketKind {
    /// The short-lived socket used for request/reply command traffic.
    Command,
    /// The subscribed socket used for compositor events.
    EventStream,
}

impl Display for SocketKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command => formatter.write_str("command"),
            Self::EventStream => formatter.write_str("event-stream"),
        }
    }
}

/// Errors produced by the Triad service.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Triad is not reachable because no socket path is available.
    #[error("triad is not running or no socket path is available")]
    TriadNotRunning,

    /// Connecting the named socket failed.
    #[error("cannot connect to triad {kind} socket")]
    IpcConnectionFailed {
        /// Which socket role the connection attempt was for.
        kind: SocketKind,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Reading or writing the socket failed.
    #[error("triad socket I/O error")]
    Io(#[from] io::Error),

    /// A JSON message could not be serialized or parsed.
    #[error("cannot parse triad JSON message")]
    JsonParse(#[from] serde_json::Error),

    /// Triad replied with `ok: false`.
    #[error("triad rejected request: {0}")]
    TriadRejected(String),

    /// Triad replied with the wrong message type.
    #[error("unexpected response for {request} request")]
    UnexpectedResponse {
        /// Name of the request that produced the mismatch.
        request: &'static str,
    },

    /// Triad closed the named socket unexpectedly.
    #[error("triad closed the {kind} socket")]
    SocketClosed {
        /// Which socket role was closed.
        kind: SocketKind,
    },
}

/// Convenience alias for results produced by this crate.
pub type Result<T> = std::result::Result<T, Error>;
