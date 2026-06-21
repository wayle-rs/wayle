//! D-Bus client proxy for shell IPC commands.
#![allow(missing_docs)]

use zbus::{Result, proxy};

/// D-Bus service name for shell IPC.
pub const SERVICE_NAME: &str = "com.wayle.Shell1";

/// D-Bus object path for shell IPC.
pub const SERVICE_PATH: &str = "/com/wayle/Shell";

#[proxy(
    interface = "com.wayle.Shell1",
    default_service = "com.wayle.Shell1",
    default_path = "/com/wayle/Shell",
    gen_blocking = false
)]
pub trait ShellIpc {
    async fn bar_hide(&self, monitor: &str) -> Result<()>;

    async fn bar_show(&self, monitor: &str) -> Result<()>;

    async fn bar_toggle(&self, monitor: &str) -> Result<()>;

    /// Advances the window switcher's selection, opening it if closed.
    async fn window_cycle_step(&self) -> Result<()>;

    /// Activates the window switcher's currently highlighted selection and
    /// closes it.
    async fn window_cycle_commit(&self) -> Result<()>;

    /// Cancels the window switcher's cycle, restoring the previously
    /// active window and closing it.
    async fn window_cycle_cancel(&self) -> Result<()>;

    #[zbus(property)]
    fn bar_hidden(&self) -> Result<Vec<String>>;

    #[zbus(property)]
    fn connectors(&self) -> Result<Vec<String>>;
}
