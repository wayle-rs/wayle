use std::sync::Arc;

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_notification::core::notification::Notification;
use wayle_widgets::watch;

use super::{CardCmd, NotificationPopupCard};

pub(super) fn spawn(sender: &ComponentSender<NotificationPopupCard>, config: &Arc<ConfigService>) {
    let notif_config = config.config().modules.notifications.clone();
    let shadow = notif_config.popup_shadow.clone();
    let urgency_bar = notif_config.popup_urgency_bar.clone();

    watch!(sender, [shadow.watch(), urgency_bar.watch()], |out| {
        let _ = out.send(CardCmd::ConfigChanged {
            shadow: shadow.get(),
            urgency_bar: urgency_bar.get(),
        });
    });
}

/// Re-renders the card when the underlying notification's displayed fields change
/// (content updated via replaces_id, or actions stripped when the owner disconnects).
/// The `Arc<Notification>` is stable for the notification's lifetime, so this stays
/// valid across in-place updates.
pub(super) fn spawn_notification(
    sender: &ComponentSender<NotificationPopupCard>,
    notification: &Arc<Notification>,
) {
    // All display facets are one atomic `view` snapshot now, so a single subscription covers
    // every displayed field (content updated via replaces_id, actions stripped on disconnect, …).
    watch!(sender, [notification.view.watch().skip(1)], |out| {
        let _ = out.send(CardCmd::NotificationChanged);
    });
}
