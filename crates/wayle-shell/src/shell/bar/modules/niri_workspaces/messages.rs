//! Message types for the [`NiriWorkspaces`] Relm4 component.
//!
//! [`NiriWorkspaces`]: super::NiriWorkspaces

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_niri::NiriService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct NiriWorkspacesInit {
    pub settings: BarSettings,
    pub niri: Arc<NiriService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum NiriWorkspacesMsg {
    LeftClick(u64),
    MiddleClick(u64),
    RightClick(u64),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum NiriWorkspacesCmd {
    WorkspacesChanged,
    ConfigChanged,
    BlinkTick,
}
