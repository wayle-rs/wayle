//! D-Bus interface for the audio service.
//!
//! Contains the server-side daemon interface and client-side proxy.

mod client;
mod server;

pub use client::AudioProxy;
pub(crate) use server::AudioDaemon;

/// D-Bus service name.
pub const SERVICE_NAME: &str = "com.wayle.Audio1";

/// D-Bus object path.
pub const SERVICE_PATH: &str = "/com/wayle/Audio";
