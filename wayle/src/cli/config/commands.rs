use std::path::PathBuf;

use clap::Subcommand;

/// Configuration management subcommands.
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Get the value of a configuration path
    Get {
        /// The configuration path to retrieve (e.g., "modules.battery.enabled")
        path: String,
    },
    /// Set the value of a configuration path
    Set {
        /// The configuration path to set (e.g., "modules.battery.enabled")
        path: String,
        /// The value to set (use JSON format for complex types)
        value: String,
    },
    /// Reset a configuration path to its default value
    Reset {
        /// The configuration path to reset (e.g., "bar.button_gap")
        path: String,
    },
    /// Output JSON Schema for the configuration (for editor intellisense)
    Schema {
        /// Print to stdout instead of writing to config directory
        #[arg(long)]
        stdout: bool,
    },
    /// Output the default configuration as TOML
    Default {
        /// Print to stdout instead of writing config.toml.example
        #[arg(long)]
        stdout: bool,
    },
    /// Generate markdown reference pages for every registered schema
    Docs {
        /// Output directory for the generated pages
        #[arg(long, default_value = "docs/config")]
        out: PathBuf,
        /// Regenerate only the named module
        #[arg(long)]
        only: Option<String>,
    },
}
