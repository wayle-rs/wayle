/// CLI application structure and parsing
pub mod app;
/// Audio control commands
pub mod audio;
/// Configuration management commands
pub mod config;
mod dbus;
/// Bar dropdown control commands
pub mod dropdown;
/// Icon management commands
pub mod icons;
/// Idle inhibit control commands
pub mod idle;
/// Media control commands
pub mod media;
/// Notification control commands
pub mod notify;
/// Panel management commands
pub mod panel;
/// Power profile commands
pub mod power;
/// ANSI styling for help output
pub mod style;
/// System tray commands
pub mod systray;
/// Wallpaper control commands
pub mod wallpaper;

/// Result type for CLI operations that return output text
pub type CliResult = Result<String, String>;
/// Result type for CLI operations that perform actions
pub type CliAction = Result<(), String>;

pub use app::{Cli, Commands};
