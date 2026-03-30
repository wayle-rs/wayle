//! Hyprland toplevel export window capture.
//!
//! Provides a background thread that captures individual window pixels via the
//! `hyprland_toplevel_export_manager_v1` Wayland protocol and returns raw BGRA
//! thumbnail data for compositing.
//!
//! Supports both one-shot snapshots and continuous streaming at a capped rate.

mod capture;

use std::sync::mpsc;
use std::time::Duration;

/// A window to capture, with its position and size relative to the monitor.
#[derive(Debug, Clone)]
pub struct CaptureClient {
    /// Hex address string (without `0x` prefix).
    pub address: String,
    /// X position relative to monitor origin.
    pub x: i32,
    /// Y position relative to monitor origin.
    pub y: i32,
    /// Logical window width.
    pub width: i32,
    /// Logical window height.
    pub height: i32,
}

/// Request to capture all windows in a workspace.
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    /// Opaque session identifier. Results carry this back so the caller can
    /// discard frames from a previous session (e.g., after close + reopen).
    pub session: u64,
    /// Workspace ID.
    pub ws_id: i64,
    /// Monitor logical width.
    pub monitor_width: u32,
    /// Monitor logical height.
    pub monitor_height: u32,
    /// Windows to capture with their positions.
    pub clients: Vec<CaptureClient>,
}

/// Commands sent to the capture thread.
#[derive(Debug, Clone)]
pub enum CaptureCommand {
    /// Capture a workspace and begin streaming frames at the capped rate.
    /// The thread will re-capture the same request periodically until a new
    /// command arrives.
    StartStreaming(CaptureRequest),
    /// Stop streaming. The thread returns to idle (blocking on next command).
    StopStreaming,
}

/// Captured pixel data for a single window.
#[derive(Debug)]
pub struct WindowThumbnail {
    /// Raw BGRA pixel data.
    pub data: Vec<u8>,
    /// Captured image width (may differ from logical size due to scaling).
    pub width: u32,
    /// Captured image height.
    pub height: u32,
    /// Bytes per row in `data`.
    pub stride: u32,
    /// Window X position relative to monitor origin.
    pub x: i32,
    /// Window Y position relative to monitor origin.
    pub y: i32,
    /// Logical window width.
    pub win_width: i32,
    /// Logical window height.
    pub win_height: i32,
    /// Hex address string (without `0x` prefix).
    pub address: String,
}

/// Result of capturing all windows in a workspace.
#[derive(Debug)]
pub struct CaptureResult {
    /// Session identifier from the originating [`CaptureRequest`].
    pub session: u64,
    /// The workspace that was captured.
    pub ws_id: i64,
    /// Captured window thumbnails.
    pub thumbnails: Vec<WindowThumbnail>,
    /// Monitor logical width.
    pub monitor_width: u32,
    /// Monitor logical height.
    pub monitor_height: u32,
}

/// Interval between streaming frames (~10 fps).
pub const STREAM_INTERVAL: Duration = Duration::from_millis(100);

/// Spawn a background thread that captures window pixels via the Hyprland
/// toplevel export Wayland protocol.
///
/// Returns a sender for submitting [`CaptureCommand`]s and a receiver for
/// collecting [`CaptureResult`]s. The thread exits when the sender is dropped.
///
/// When streaming, the thread re-captures the current request at approximately
/// [`STREAM_INTERVAL`] rate, skipping frames if the previous capture is still
/// in progress (captures are serial, never overlapping).
pub fn spawn_capture_thread() -> (mpsc::Sender<CaptureCommand>, mpsc::Receiver<CaptureResult>) {
    capture::spawn()
}
