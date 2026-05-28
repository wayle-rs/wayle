//! Init parameters, input messages, and command outputs for the
//! keyboard-input component.

use std::{rc::Rc, sync::Arc};

use wayle_config::ConfigService;
use wayle_widgets::prelude::BarSettings;

use super::sources::{CurrentLayout, KeyboardLayoutSource};
use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct KeyboardInputInit {
    pub settings: BarSettings,
    pub source: Arc<dyn KeyboardLayoutSource>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum KeyboardInputMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
pub(crate) enum KeyboardInputCmd {
    LayoutChanged(Option<CurrentLayout>),
    FormatChanged,
    LayoutAliasMapChanged,
    UpdateIcon(String),
}
