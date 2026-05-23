//! Message types for the [`TriadWorkspaces`] Relm4 component.
//!
//! [`TriadWorkspaces`]: super::TriadWorkspaces

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_triad::TriadService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct TriadWorkspacesInit {
    pub settings: BarSettings,
    pub triad: Arc<TriadService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum TriadWorkspacesMsg {
    LeftClick(u64),
    MiddleClick(u64),
    RightClick(u64),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum TriadWorkspacesCmd {
    WorkspacesChanged,
    ConfigChanged,
    BlinkTick,
}
