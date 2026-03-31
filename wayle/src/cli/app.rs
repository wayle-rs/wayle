use clap::{
    CommandFactory, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use clap_complete::Shell;

use crate::cli::{
    audio::commands::AudioCommands, config::commands::ConfigCommands,
    icons::commands::IconsCommands, idle::commands::IdleCommands, media::commands::MediaCommands,
    notify::commands::NotifyCommands, panel::commands::PanelCommands,
    power::commands::PowerCommands, systray::commands::SystrayCommands,
    wallpaper::commands::WallpaperCommands,
};

fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
        .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .placeholder(AnsiColor::Green.on_default())
        .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
        .valid(AnsiColor::Green.on_default().effects(Effects::BOLD))
        .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
}

/// Wayle - A Wayland compositor agnostic shell
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(styles = get_styles())]
pub struct Cli {
    /// The command to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Audio control commands
    Audio {
        /// Audio subcommand to execute.
        #[command(subcommand)]
        command: AudioCommands,
    },
    /// Configuration management commands
    Config {
        /// Configuration subcommand to execute.
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Icon management commands
    Icons {
        /// Icons subcommand to execute.
        #[command(subcommand)]
        command: IconsCommands,
    },
    /// Media player control commands
    Media {
        /// Media subcommand to execute.
        #[command(subcommand)]
        command: MediaCommands,
    },
    /// Notification control commands
    Notify {
        /// Notification subcommand to execute.
        #[command(subcommand)]
        command: NotifyCommands,
    },
    /// Panel management commands
    Panel {
        /// Panel subcommand to execute.
        #[command(subcommand)]
        command: PanelCommands,
    },
    /// Power profile commands
    Power {
        /// Power subcommand to execute.
        #[command(subcommand)]
        command: PowerCommands,
    },
    /// System tray commands
    Systray {
        /// Systray subcommand to execute.
        #[command(subcommand)]
        command: SystrayCommands,
    },
    /// Wallpaper control commands
    Wallpaper {
        /// Wallpaper subcommand to execute.
        #[command(subcommand)]
        command: WallpaperCommands,
    },
    /// Idle inhibit control commands
    Idle {
        /// Idle subcommand to execute.
        #[command(subcommand)]
        command: IdleCommands,
    },
    /// Run the desktop shell in the foreground
    Shell,
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for.
        shell: Shell,
    },
}

/// Prints shell completions to stdout.
pub fn generate_completions(shell: Shell) {
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        "wayle",
        &mut std::io::stdout(),
    );
}
