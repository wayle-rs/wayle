//! Message types for the [`MangoWorkspaces`] Relm4 component.
//!
//! [`MangoWorkspaces`]: super::MangoWorkspaces

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_mango::MangoService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct MangoWorkspacesInit {
    pub settings: BarSettings,
    pub mango: Arc<MangoService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum MangoWorkspacesMsg {
    LeftClick(u32),
    MiddleClick(u32),
    RightClick(u32),
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum MangoWorkspacesCmd {
    TagsChanged,
    ConfigChanged,
    BlinkTick,
}
