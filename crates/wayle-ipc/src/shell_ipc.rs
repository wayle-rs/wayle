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

    async fn dropdown_list(&self, monitor: &str) -> Result<Vec<String>>;

    async fn dropdown_toggle(&self, monitor: &str, identifier: &str) -> Result<()>;

    async fn dropdown_open(&self, monitor: &str, identifier: &str) -> Result<()>;

    async fn dropdown_close(&self, monitor: &str) -> Result<()>;

    async fn systray_toggle(&self, id: &str, monitor: &str) -> Result<()>;

    async fn systray_open(&self, id: &str, monitor: &str) -> Result<()>;

    #[zbus(property)]
    fn bar_hidden(&self) -> Result<Vec<String>>;

    #[zbus(property)]
    fn connectors(&self) -> Result<Vec<String>>;
}
