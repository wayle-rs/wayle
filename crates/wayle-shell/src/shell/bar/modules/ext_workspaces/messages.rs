//! Message types for the [`ExtWorkspaces`] Relm4 component.
//!
//! [`ExtWorkspaces`]: super::ExtWorkspaces

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_ext_workspace::ExtWorkspaceService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct ExtWorkspacesInit {
    pub settings: BarSettings,
    pub service: Arc<ExtWorkspaceService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum ExtWorkspacesMsg {
    LeftClick(u32),
    MiddleClick(u32),
    RightClick(u32),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum ExtWorkspacesCmd {
    WorkspacesChanged,
    ConfigChanged,
}
