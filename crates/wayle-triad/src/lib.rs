//! Native Triad IPC bindings for Wayle.

mod error;
mod ipc;
mod service;
mod types;

pub use error::{Error, Result, SocketKind};
pub use service::TriadService;
pub use types::{
    Capabilities, Geometry, KeyboardLayoutTarget, KeyboardLayouts, Output, Point, Size, TriadEvent,
    Window, WindowPosition, Workspace,
};
