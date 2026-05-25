use std::path::PathBuf;

use clap::Subcommand;

use crate::styled_header;

/// Icon management subcommands.
#[derive(Subcommand, Debug)]
pub enum IconsCommands {
    /// Install bundled icons required by Wayle components
    Setup,
    /// Install icons from a CDN source
    #[command(after_long_help = INSTALL_HELP)]
    Install {
        /// Source name (run 'wayle icons sources' to see available sources)
        source: String,
        /// Icon slugs to install (e.g., home settings bell)
        #[arg(required = true)]
        slugs: Vec<String>,
    },
    /// Import local SVG file(s) as icons
    #[command(after_long_help = IMPORT_HELP)]
    Import {
        /// Path to SVG file or directory
        path: PathBuf,
        /// Icon name (required for single file, ignored for directory)
        name: Option<String>,
    },
    /// Remove installed icons
    Remove {
        /// Icon names to remove (e.g., tb-home-symbolic si-firefox-symbolic)
        #[arg(required = true)]
        names: Vec<String>,
    },
    /// List available icon sources
    Sources,
    /// List installed icons
    List {
        /// Filter by source prefix (e.g., tb, si, md)
        #[arg(short, long)]
        source: Option<String>,
        /// Interactive fuzzy search (requires fzf)
        #[arg(short, long)]
        interactive: bool,
    },
    /// Open the icons directory in file manager
    Open,
    /// Export all installed icons to a directory
    Export {
        /// Destination directory for exported icons
        destination: PathBuf,
    },
    /// Install any icons referenced in config but not yet on disk
    #[command(after_long_help = SYNC_HELP)]
    Sync {
        /// Preview what would be installed without making changes
        #[arg(long)]
        dry_run: bool,
    },
}

const INSTALL_HELP: &str = concat!(
    styled_header!("Examples:"),
    "\n",
    "    wayle icons install tabler home settings bell\n",
    "        -> tb-home-symbolic, tb-settings-symbolic, tb-bell-symbolic\n",
    "\n",
    "    wayle icons install simple-icons firefox spotify\n",
    "        -> si-firefox-symbolic, si-spotify-symbolic\n",
    "\n",
    "Run 'wayle icons sources' to see all available icon sources.\n",
    "Icons are saved to ~/.local/share/wayle/icons/ as GTK symbolic icons.",
);

const IMPORT_HELP: &str = concat!(
    styled_header!("Examples:"),
    "\n",
    "    wayle icons import ~/Downloads/my-icon.svg my-icon\n",
    "        -> cm-my-icon-symbolic\n",
    "\n",
    "    wayle icons import ~/exported-icons/\n",
    "        -> Imports all SVGs, preserves names with known prefixes\n",
    "\n",
    "Icons without a known prefix (tb-, tbf-, si-, md-, ld-) get 'cm-' added.",
);

const SYNC_HELP: &str = concat!(
    styled_header!("Examples:"),
    "\n",
    "    wayle icons sync\n",
    "        -> install every config-referenced icon not yet on disk\n",
    "\n",
    "    wayle icons sync --dry-run\n",
    "        -> list what would be installed; no downloads\n",
    "\n",
    "Useful when sharing dotfiles across machines. User-imported `cm-` icons\n",
    "cannot be auto-installed and are listed as skipped.",
);
