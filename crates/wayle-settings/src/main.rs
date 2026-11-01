//! Wayle Settings GUI.
//!
//! Separate binary from the shell. Reads the same config files,
//! writes runtime overrides to runtime.toml, and the shell picks
//! up changes via its file watcher. Closes cleanly when the window
//! is dismissed, freeing all RAM.

mod app;
mod editors;
mod pages;
mod property_handle;
mod row;
mod sidebar;

use std::{env, process};

use tokio::runtime::Runtime;
use tracing_subscriber::EnvFilter;
use wayle_config::{ConfigService, PersistenceWatcher};

/// Forces GTK's OpenGL renderer before GTK initializes.
///
/// # Safety
///
/// `set_var` races with other threads reading the environment. This is called
/// at the top of `main` before the tokio runtime and GTK are constructed, so
/// the only readers are libc internals; racing there is still a hazard on
/// some glibc versions, but the window is small enough to accept for a short
/// -lived GUI binary.
#[allow(unsafe_code)]
unsafe fn force_gl_renderer() {
    unsafe {
        env::set_var("GSK_RENDERER", "gl");
    }
}

fn main() {
    if env::var_os("GSK_RENDERER").is_none() {
        #[allow(unsafe_code)]
        unsafe {
            force_gl_renderer();
        }
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let runtime = match Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("cannot create tokio runtime: {err}");
            process::exit(1);
        }
    };

    let _guard = runtime.enter();

    let config_service = match runtime.block_on(ConfigService::load()) {
        Ok(service) => service,
        Err(err) => {
            eprintln!("cannot load config: {err}");
            process::exit(1);
        }
    };

    let _persistence = match PersistenceWatcher::start(config_service.clone()) {
        Ok(watcher) => watcher,
        Err(err) => {
            eprintln!("cannot start persistence watcher: {err}");
            process::exit(1);
        }
    };

    let relm_app = relm4::RelmApp::new("com.wayle.settings");

    relm_app.run::<app::SettingsApp>(config_service);
}
