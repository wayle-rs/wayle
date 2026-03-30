//! Wayle CLI entry point.
//!
//! Parses CLI args and dispatches to the appropriate handler.
//! The `shell` subcommand runs the GUI directly and manages its own
//! tokio runtime. All other commands share a single runtime.

use std::process;

use clap::Parser;
use tokio::runtime::Runtime;
use wayle::{
    cli::{self, Cli, Commands},
    core::{init, tracing as tracing_init},
};

fn main() {
    let cli = Cli::parse();

    if matches!(cli.command, Commands::Shell) {
        return run_shell();
    }

    let Ok(runtime) = Runtime::new() else {
        eprintln!("Failed to create tokio runtime");
        process::exit(1);
    };

    let result = runtime.block_on(async {
        if let Err(err) = tracing_init::init_cli_mode() {
            eprintln!("Failed to initialize tracing: {err}");
        }

        if let Err(err) = init::ensure_directories() {
            eprintln!("Failed to ensure directories: {err}");
        }

        match cli.command {
            Commands::Audio { command } => cli::audio::execute(command).await,
            Commands::Config { command } => cli::config::execute(command).await,
            Commands::Icons { command } => cli::icons::execute(command).await,
            Commands::Media { command } => cli::media::execute(command).await,
            Commands::Notify { command } => cli::notify::execute(command).await,
            Commands::Panel { command } => cli::panel::execute(command).await,
            Commands::Power { command } => cli::power::execute(command).await,
            Commands::Systray { command } => cli::systray::execute(command).await,
            Commands::Wallpaper { command } => cli::wallpaper::execute(command).await,
            Commands::Idle { command } => cli::idle::execute(command).await,
            Commands::Shell => unreachable!(),
        }
    });

    if let Err(err) = result {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run_shell() {
    if let Err(err) = wayle_shell::run() {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}
