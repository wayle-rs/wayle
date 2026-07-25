use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_notification::{NotificationService, core::notification::Notification};

pub(crate) struct NotificationDropdownInit {
    pub notification: Arc<NotificationService>,
    pub config: Arc<ConfigService>,
}

#[derive(Debug)]
pub(crate) enum NotificationDropdownMsg {
    DndToggled(bool),
    ClearAll,
    ClearGroup(Vec<Arc<Notification>>),
}

#[derive(Debug)]
pub(crate) enum NotificationDropdownCmd {
    NotificationsChanged,
    DndChanged(bool),
    ScaleChanged(f32),
    IconSourceChanged,
    TimeTick,
}
