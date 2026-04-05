//! Wayle desktop shell - a GTK4/Relm4 status bar for Wayland compositors.

use std::{env, error::Error};

use relm4::RelmApp;
use tokio::runtime::Runtime;
use tracing::info;

mod bootstrap;
mod glob;
mod i18n;
mod process;
mod services;
mod shell;
mod startup;
mod template;
mod tracing_init;
mod wallpaper_map;
mod watchers;

use shell::{Shell, ShellInit};

/// Launches the Wayle shell GUI.
///
/// Creates its own tokio runtime internally, so this must not be called
/// from within an existing tokio context (it will panic).
///
/// # Errors
///
/// Returns error on tracing init failure, runtime creation failure,
/// or service bootstrap failure.
pub fn run() -> Result<(), Box<dyn Error>> {
    if env::var_os("GSK_RENDERER").is_none() {
        #[allow(unsafe_code)]
        // SAFETY: single-threaded, called before any runtime or GTK init
        unsafe {
            env::set_var("GSK_RENDERER", "gl");
        }
    }

    tracing_init::init()?;
    info!("Wayle shell starting");

    let runtime = Runtime::new()?;
    let _guard = runtime.enter();

    if runtime.block_on(bootstrap::is_already_running()) {
        eprintln!("Wayle shell is already running");
        return Ok(());
    }

    let (timer, services) = runtime.block_on(bootstrap::init_services())?;
    info!("Services initialized");

    let app = RelmApp::new("com.wayle.shell")
        .visible_on_activate(false)
        .with_args(vec![]);

    app.run::<Shell>(ShellInit { timer, services });

    info!("Wayle shell stopped");

    runtime.shutdown_background();
    Ok(())
}
