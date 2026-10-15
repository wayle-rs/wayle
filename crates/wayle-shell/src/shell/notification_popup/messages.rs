use std::sync::Arc;

use wayle_config::ConfigService;
use wayle_notification::{NotificationService, core::notification::Notification};
use wayle_output::OutputService;

/// Initialization data for the notification popup host.
pub(crate) struct PopupHostInit {
    pub(crate) notification: Arc<NotificationService>,
    pub(crate) config: Arc<ConfigService>,
    pub(crate) outputs: Option<Arc<OutputService>>,
}

/// Commands for popup host updates.
#[derive(Debug)]
pub(crate) enum PopupHostCmd {
    PopupsChanged(Vec<Arc<Notification>>),
    ConfigChanged,
}
