use std::{sync::Arc, time::Duration};

use relm4::ComponentSender;
use wayle_config::ConfigService;
use wayle_notification::NotificationService;
use wayle_widgets::watch;

use super::{NotificationDropdown, messages::NotificationDropdownCmd};

const TIME_TICK_INTERVAL: Duration = Duration::from_secs(30);

pub(super) fn spawn(
    sender: &ComponentSender<NotificationDropdown>,
    notification: &Arc<NotificationService>,
    config: &Arc<ConfigService>,
) {
    spawn_notifications_watcher(sender, notification);
    spawn_dnd_watcher(sender, notification);
    spawn_scale_watcher(sender, config);
    spawn_icon_source_watcher(sender, config);
    spawn_relative_time_refresh(sender);
}

fn spawn_relative_time_refresh(sender: &ComponentSender<NotificationDropdown>) {
    sender.command(|out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                () = tokio::time::sleep(TIME_TICK_INTERVAL) => {
                    let _ = out.send(NotificationDropdownCmd::TimeTick);
                }
            }
        }
    });
}

fn spawn_notifications_watcher(
    sender: &ComponentSender<NotificationDropdown>,
    notification: &Arc<NotificationService>,
) {
    let notifications_prop = notification.notifications.clone();

    watch!(sender, [notifications_prop.watch()], |out| {
        let _ = out.send(NotificationDropdownCmd::NotificationsChanged);
    });
}

fn spawn_dnd_watcher(
    sender: &ComponentSender<NotificationDropdown>,
    notification: &Arc<NotificationService>,
) {
    let dnd_prop = notification.dnd.clone();

    watch!(sender, [dnd_prop.watch()], |out| {
        let _ = out.send(NotificationDropdownCmd::DndChanged(dnd_prop.get()));
    });
}

fn spawn_icon_source_watcher(
    sender: &ComponentSender<NotificationDropdown>,
    config: &Arc<ConfigService>,
) {
    let icon_source = config.config().modules.notifications.icon_source.clone();

    watch!(sender, [icon_source.watch()], |out| {
        let _ = out.send(NotificationDropdownCmd::IconSourceChanged);
    });
}

fn spawn_scale_watcher(
    sender: &ComponentSender<NotificationDropdown>,
    config: &Arc<ConfigService>,
) {
    let scale = config.config().styling.scale.clone();

    watch!(sender, [scale.watch()], |out| {
        let _ = out.send(NotificationDropdownCmd::ScaleChanged(scale.get().value()));
    });
}
