//! Spawn compositor event subscription for dock running-apps tracking.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use wayle_hyprland::{HyprlandEvent, HyprlandService};
use wayle_niri::{Event as NiriEvent, NiriService};

use super::adapter::DockAdapterRef;
use super::Dock;
use super::DockCmd;

/// Spawn an async task that subscribes to compositor events and
/// triggers DockCmd::DockItemsChanged when running apps change.
pub(crate) fn spawn(
    sender: &ComponentSender<Dock>,
    _services: &crate::shell::services::ShellServices,
    adapter: DockAdapterRef,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    match &adapter {
        DockAdapterRef::Niri(niri) => {
            spawn_niri_watcher(niri.niri.clone(), tx);
        }
        DockAdapterRef::Hyprland(hyprland) => {
            spawn_hyprland_watcher(hyprland.hyprland.clone(), tx);
        }
    }

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(_) = rx.recv() => {
                    let _ = out.send(DockCmd::DockItemsChanged);
                }
            }
        }
    });
}

fn spawn_niri_watcher(
    niri: Arc<NiriService>,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    tokio::spawn(async move {
        let mut events = niri.events();
        let mut windows_changed = niri.windows.watch();

        loop {
            tokio::select! {
                event = events.next() => {
                    if let Some(event) = event {
                        if needs_niri_update(&event) {
                            let _ = tx.send(());
                        }
                    } else {
                        break;
                    }
                }
                Some(_) = windows_changed.next() => {
                    let _ = tx.send(());
                }
            }
        }
    });
}

fn spawn_hyprland_watcher(
    hyprland: Arc<HyprlandService>,
    tx: tokio::sync::mpsc::UnboundedSender<()>,
) {
    tokio::spawn(async move {
        let mut events = hyprland.events();
        let mut clients_changed = hyprland.clients.watch();

        loop {
            tokio::select! {
                event = events.next() => {
                    if let Some(event) = event {
                        if needs_hyprland_update(&event) {
                            let _ = tx.send(());
                        }
                    } else {
                        break;
                    }
                }
                Some(_) = clients_changed.next() => {
                    let _ = tx.send(());
                }
            }
        }
    });
}

fn needs_niri_update(event: &NiriEvent) -> bool {
    matches!(
        event,
        NiriEvent::WindowsChanged { .. }
            | NiriEvent::WindowOpenedOrChanged { .. }
            | NiriEvent::WindowClosed { .. }
            | NiriEvent::WindowFocusChanged { .. }
            | NiriEvent::WindowLayoutsChanged { .. }
    )
}

fn needs_hyprland_update(event: &HyprlandEvent) -> bool {
    matches!(
        event,
        HyprlandEvent::OpenWindow { .. }
            | HyprlandEvent::CloseWindow { .. }
            | HyprlandEvent::MoveWindow { .. }
            | HyprlandEvent::MoveWindowV2 { .. }
            | HyprlandEvent::ActiveWindow { .. }
            | HyprlandEvent::ActiveWindowV2 { .. }
            | HyprlandEvent::Minimized { .. }
    )
}
