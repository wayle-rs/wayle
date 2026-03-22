use std::{rc::Rc, sync::Arc};

use wayle_config::{ConfigService, schemas::styling::ThresholdColors};
use wayle_notification::NotificationService;
use wayle_widgets::prelude::BarSettings;

use crate::shell::bar::dropdowns::DropdownRegistry;

pub(crate) struct NotificationInit {
    pub settings: BarSettings,
    pub notification: Arc<NotificationService>,
    pub config: Arc<ConfigService>,
    pub dropdowns: Rc<DropdownRegistry>,
}

#[derive(Debug)]
pub(crate) enum NotificationMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    ScrollUp,
    ScrollDown,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum NotificationCmd {
    NotificationsChanged(usize),
    DndChanged(bool),
    IconConfigChanged,
    UpdateThresholdColors(ThresholdColors),
}
