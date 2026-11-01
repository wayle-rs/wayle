use std::{cmp::Reverse, collections::HashMap, sync::Arc};

use wayle_notification::core::notification::Notification;

pub(super) struct NotificationGroupData {
    pub app_name: Option<String>,
    pub notifications: Vec<Arc<Notification>>,
}

/// Groups a flat notification list by app name.
///
/// Groups are ordered by most recent notification timestamp (newest first).
/// Within each group, notifications are ordered newest first.
pub(super) fn group_by_app(notifications: &[Arc<Notification>]) -> Vec<NotificationGroupData> {
    let mut groups: HashMap<Option<String>, Vec<Arc<Notification>>> = HashMap::new();

    for notification in notifications {
        let key = notification.app_name.get();
        groups.entry(key).or_default().push(notification.clone());
    }

    let mut result: Vec<NotificationGroupData> = groups
        .into_iter()
        .map(|(app_name, mut notifs)| {
            notifs.sort_by_key(|notification| Reverse(notification.timestamp.get()));
            NotificationGroupData {
                app_name,
                notifications: notifs,
            }
        })
        .collect();

    result.sort_by(|left, right| {
        let left_ts = left
            .notifications
            .first()
            .map(|notification| notification.timestamp.get());
        let right_ts = right
            .notifications
            .first()
            .map(|notification| notification.timestamp.get());
        right_ts.cmp(&left_ts)
    });

    result
}
