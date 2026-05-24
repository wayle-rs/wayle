//! Init parameters, input messages, and command outputs for the
//! window-title component.

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;

use super::sources::{FocusedWindow, FocusedWindowSource};
use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct WindowTitleInit {
    pub settings: BarSettings,
    pub source: Arc<dyn FocusedWindowSource>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum WindowTitleMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum WindowTitleCmd {
    WindowChanged {
        focused: Option<FocusedWindow>,
        format: String,
    },
    FormatChanged,
    IconConfigChanged,
}
