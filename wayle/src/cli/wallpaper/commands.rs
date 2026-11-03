use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

/// Wallpaper control subcommands.
#[derive(Subcommand, Debug)]
pub enum WallpaperCommands {
    /// Set wallpaper from an image file
    Set {
        /// Path to wallpaper image
        path: PathBuf,
        /// Image fit mode
        #[arg(short, long, value_enum)]
        fit: Option<FitModeArg>,
        /// Target monitor: connector (DP-1), "desc:<prefix>" or "sn:<serial>". If omitted, applies to all monitors.
        #[arg(long)]
        monitor: Option<String>,
    },

    /// Start cycling wallpapers from a directory
    Cycle {
        /// Directory containing wallpaper images
        directory: PathBuf,
        /// Interval in seconds between changes
        #[arg(short, long, default_value = "300")]
        interval: u32,
        /// Cycling mode
        #[arg(short = 'm', long, value_enum, default_value = "sequential")]
        mode: CyclingModeArg,
    },

    /// Stop wallpaper cycling
    Stop,

    /// Skip to next wallpaper
    Next,

    /// Go back to previous wallpaper
    Previous,

    /// Display current wallpaper information
    Info {
        /// Target monitor (e.g., DP-1, HDMI-A-1). If omitted, shows global state.
        #[arg(long)]
        monitor: Option<String>,
    },

    /// Set which monitor to use for color extraction
    ThemingMonitor {
        /// Monitor connector name (e.g., DP-1). Use empty string for default.
        monitor: String,
    },
}

/// Image fit mode for CLI.
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum FitModeArg {
    /// Scale to cover entire display
    Fill,
    /// Scale to fit within display
    Fit,
    /// Display at original size, centered
    Center,
    /// Tile the image
    Tile,
    /// Stretch to fill
    Stretch,
}

/// Cycling mode for CLI.
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum CyclingModeArg {
    /// Cycle in alphabetical order
    Sequential,
    /// Cycle in random order
    Shuffle,
}
