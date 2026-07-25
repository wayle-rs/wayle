use std::sync::Arc;

use wayle_config::schemas::modules::notification::IconSource;
use wayle_notification::core::notification::Notification;

pub(crate) struct NotificationGroupInit {
    pub app_name: Option<String>,
    pub notifications: Vec<Arc<Notification>>,
    pub icon_source: IconSource,
    pub prefer_color: bool,
}

#[derive(Debug)]
pub(crate) enum NotificationGroupInput {
    ToggleExpanded,
    ShowAll,
    ClearGroup,
    UpdateNotifications(Vec<Arc<Notification>>),
    RefreshTime,
    ItemDismissed(u32),
}

#[derive(Debug)]
pub(crate) enum NotificationGroupOutput {
    Dismissed,
}
