use std::sync::Arc;

use wayle_config::schemas::modules::notification::IconSource;
use wayle_notification::core::notification::Notification;

pub(crate) struct NotificationGroupInit {
    pub app_name: Option<String>,
    pub notifications: Vec<Arc<Notification>>,
    pub icon_source: IconSource,
}

#[derive(Debug)]
pub(crate) enum NotificationGroupInput {
    ToggleExpanded,
    ShowAll,
    ClearGroup,
    UpdateNotifications(Vec<Arc<Notification>>),
    RefreshTime,
}

#[derive(Debug)]
pub(crate) enum NotificationGroupOutput {
    /// Requests the parent (which owns the service) to clear these notifications as a
    /// single batch.
    ClearRequested(Vec<Arc<Notification>>),
}
