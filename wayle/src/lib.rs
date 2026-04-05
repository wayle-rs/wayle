//! Wayle CLI - Compositor-agnostic desktop environment CLI.
//!
//! CLI commands for managing Wayle services:
//!
//! - `wayle panel` - Start/stop/control the panel GUI
//! - `wayle media` - Control media players
//! - `wayle wallpaper` - Manage wallpapers
//! - `wayle config` - Query/set configuration
//! - `wayle icons` - Manage icon packs
//!
//! The GUI panel runs via `wayle shell` (or `wayle panel start` for daemon mode).

/// Configuration schema definitions and validation.
pub use wayle_config as config;

/// Documentation generation for configuration schemas.
pub mod docs;

/// Command-line interface.
pub mod cli;

/// Core runtime infrastructure.
pub mod core;
