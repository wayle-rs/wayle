//! Notification blocklist hot-reload watcher.

use std::sync::Arc;

use futures::StreamExt;
use wayle_notification::NotificationService;

use crate::shell::ShellServices;

/// Syncs the notification blocklist from config to the service on change.
pub fn spawn(services: &ShellServices) {
    let notification_enabled = services.config.config().modules.notifications.enabled.get();
    let Some(notification) = &services.notification else {
        return;
    };

    if !notification_enabled {
        return;
    }

    let config = services.config.config();
    spawn_blocklist_watcher(&config.modules.notifications, notification);
}

fn spawn_blocklist_watcher(
    config: &wayle_config::schemas::modules::notification::NotificationConfig,
    service: &Arc<NotificationService>,
) {
    let mut stream = config.blocklist.watch();
    let service = service.clone();

    tokio::spawn(async move {
        stream.next().await;

        while let Some(patterns) = stream.next().await {
            service.set_blocklist(patterns);
        }
    });
}
