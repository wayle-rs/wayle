use std::{rc::Rc, sync::Arc};

use wayle_config::{ConfigProperty, ConfigService};
use wayle_systray::{SystemTrayService, core::item::TrayItem};

use crate::{
    services::shell_ipc::{ShellIpcState, SystrayMenuAction},
    shell::bar::dropdowns::OpenSurfaceCoordinator,
};

pub(crate) struct SystrayInit {
    pub is_vertical: ConfigProperty<bool>,
    pub systray: Arc<SystemTrayService>,
    pub config: Arc<ConfigService>,
    pub coordinator: Rc<OpenSurfaceCoordinator>,
    pub shell_ipc: ShellIpcState,
    /// This bar's connector, so `wayle systray toggle/open --monitor` can target it.
    pub monitor: Option<String>,
}

#[derive(Debug)]
pub(crate) enum SystrayMsg {}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum SystrayCmd {
    ItemsChanged(Vec<Arc<TrayItem>>),
    StylingChanged,
    OrientationChanged(bool),
    /// A `wayle systray toggle`/`open <id>` request: act on the matching item's menu.
    MenuRequest(SystrayMenuAction, String),
}
