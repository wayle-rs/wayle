use std::sync::Arc;

use wayle_config::ConfigService;

pub(crate) struct UpdatesDropdownInit {
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum UpdatesDropdownInput {
    Refresh,
    UpdateAll,
}

#[derive(Debug)]
pub(crate) enum UpdatesDropdownCmd {
    ScaleChanged(f32),
    UpdateCounts { pacman: u32, aur: u32, flatpak: u32 },
    SetChecking(bool),
}
