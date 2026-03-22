use std::sync::Arc;

use relm4::ComponentSender;
use wayle_common::watch;
use wayle_config::schemas::{modules::NotificationConfig, styling::evaluate_thresholds};
use wayle_notification::NotificationService;

use super::{NotificationModule, messages::NotificationCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<NotificationModule>,
    config: &NotificationConfig,
    notification: &Arc<NotificationService>,
) {
    let thresholds = config.thresholds.clone();

    let notifications = notification.notifications.clone();
    watch!(sender, [notifications.watch()], |out| {
        let count = notifications.get().len();
        let _ = out.send(NotificationCmd::NotificationsChanged(count));

        let colors = evaluate_thresholds(count as f64, &thresholds.get());
        let _ = out.send(NotificationCmd::UpdateThresholdColors(colors));
    });

    let dnd = notification.dnd.clone();
    watch!(sender, [dnd.watch()], |out| {
        let _ = out.send(NotificationCmd::DndChanged(dnd.get()));
    });

    let icon_name = config.icon_name.clone();
    let icon_unread = config.icon_unread.clone();
    let icon_dnd = config.icon_dnd.clone();
    watch!(
        sender,
        [icon_name.watch(), icon_unread.watch(), icon_dnd.watch()],
        |out| {
            let _ = out.send(NotificationCmd::IconConfigChanged);
        }
    );
}
