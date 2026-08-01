use std::sync::Arc;

use futures::{StreamExt, stream::select};
use relm4::prelude::FactorySender;
use tokio_util::sync::CancellationToken;
use wayle_systray::core::item::TrayItem;

use super::{SystrayItem, SystrayItemMsg};

pub(super) fn spawn_menu_watcher(
    sender: &FactorySender<SystrayItem>,
    item: &Arc<TrayItem>,
    cancel_token: CancellationToken,
) {
    // No `skip(1)`: the initial layout must fire `MenuUpdated` too, so the cascade is
    // pre-built off the click path as soon as the menu arrives (a click then only
    // shows it). `rebuild_cached_menu` no-ops when the layout is unchanged.
    let stream = item.menu.watch();
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
                    // Non-panicking send: this task can outlive the component during
                    // factory teardown (the runtime's input receiver is dropped before
                    // our `cancel_token` fires), so `sender.input` — which `expect`s —
                    // would panic on a queued emission. Dropping into a shut-down
                    // runtime is a no-op.
                    let _ = sender.input_sender().send(SystrayItemMsg::MenuUpdated);
                }
            }
        }
    });
}

pub(super) fn spawn_icon_watcher(
    sender: &FactorySender<SystrayItem>,
    item: &Arc<TrayItem>,
    cancel_token: CancellationToken,
) {
    let icon_name = item.icon_name.watch().skip(1).map(|_| ());
    let icon_pixmap = item.icon_pixmap.watch().skip(1).map(|_| ());
    let stream = select(icon_name, icon_pixmap);
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
                    // Non-panicking send; see `spawn_menu_watcher`.
                    let _ = sender.input_sender().send(SystrayItemMsg::IconUpdated);
                }
            }
        }
    });
}
