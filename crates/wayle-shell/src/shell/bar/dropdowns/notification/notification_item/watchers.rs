use std::sync::Arc;

use futures::StreamExt;
use relm4::prelude::FactorySender;
use tokio_util::sync::CancellationToken;
use wayle_notification::core::notification::Notification;

use super::{NotificationItem, messages::NotificationItemInput};

/// Reactively forwards any change to the notification's displayed fields as a single
/// `Refresh` input, so the item re-renders in place. All facets now live in one atomic
/// `view` snapshot, so a single subscription covers every displayed field (the
/// `Arc<Notification>` is stable across `replaces_id` updates, so this stays valid).
pub(super) fn spawn_field_watcher(
    sender: &FactorySender<NotificationItem>,
    notification: &Arc<Notification>,
    cancel_token: CancellationToken,
) {
    let stream = notification.view.watch().skip(1).map(|_| ());
    let sender = sender.clone();

    relm4::spawn_local(async move {
        futures::pin_mut!(stream);

        loop {
            tokio::select! {
                () = cancel_token.cancelled() => break,
                result = stream.next() => {
                    if result.is_none() {
                        break;
                    }
                    sender.input(NotificationItemInput::Refresh);
                }
            }
        }
    });
}
