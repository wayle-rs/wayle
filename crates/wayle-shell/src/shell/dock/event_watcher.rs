//! Spawn compositor event subscription for dock running-apps tracking.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use wayle_hyprland::{HyprlandEvent, HyprlandService};
use wayle_niri::{Event as NiriEvent, NiriService};

use super::adapter::DockAdapterRef;
use super::{Dock, DockCmd, DockEvent};

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
                Some(evt) = rx.recv() => {
                    tracing::debug!("dock event: {:?}", evt);
                    let _ = out.send(DockCmd::DockItemsChangedWithEvent(evt));
                }
            }
        }
    });
}

fn spawn_niri_watcher(
    niri: Arc<NiriService>,
    tx: tokio::sync::mpsc::UnboundedSender<DockEvent>,
) {
    let niri = Arc::new(niri);
    tokio::spawn(async move {
        let mut events = niri.events();
        let mut windows_changed = niri.windows.watch();
        let niri = niri.clone();

        loop {
            tokio::select! {
                event = events.next() => {
                    if let Some(event) = event {
                        if let Some(dock_event) = niri_event_to_dock_event(&event, &niri) {
                            let _ = tx.send(dock_event);
                        }
                    } else {
                        break;
                    }
                }
                Some(_) = windows_changed.next() => {
                    let _ = tx.send(DockEvent::WindowsChanged);
                }
            }
        }
    });
}

fn spawn_hyprland_watcher(
    hyprland: Arc<HyprlandService>,
    tx: tokio::sync::mpsc::UnboundedSender<DockEvent>,
) {
    let hyprland = Arc::new(hyprland);
    tokio::spawn(async move {
        let mut events = hyprland.events();
        let mut clients_changed = hyprland.clients.watch();
        let hyprland = hyprland.clone();

        loop {
            tokio::select! {
                event = events.next() => {
                    if let Some(event) = event {
                        if let Some(dock_event) = hyprland_event_to_dock_event(&event, &hyprland) {
                            let _ = tx.send(dock_event);
                        }
                    } else {
                        break;
                    }
                }
                Some(_) = clients_changed.next() => {
                    let _ = tx.send(DockEvent::WindowsChanged);
                }
            }
        }
    });
}

fn niri_event_to_dock_event(
    event: &NiriEvent,
    niri: &Arc<NiriService>,
) -> Option<DockEvent> {
    match event {
        NiriEvent::WindowOpenedOrChanged { .. } => Some(DockEvent::WindowOpened),
        NiriEvent::WindowClosed { .. } => Some(DockEvent::WindowClosed),
        NiriEvent::WindowFocusChanged { .. }
        | NiriEvent::WindowLayoutsChanged { .. } => {
            let focused_id = niri.focused_window_id.get();
            let focused_app = focused_id.and_then(|fid| {
                niri.windows.get().iter()
                    .find(|(_, w)| w.id.get() == fid)
                    .and_then(|(_, w)| w.app_id.get())
            });
            Some(DockEvent::ActiveWindowChanged(focused_app))
        }
        NiriEvent::WindowsChanged { .. } => Some(DockEvent::WindowsChanged),
        _ => None,
    }
}

fn hyprland_event_to_dock_event(
    event: &HyprlandEvent,
    hyprland: &Arc<HyprlandService>,
) -> Option<DockEvent> {
    match event {
        HyprlandEvent::OpenWindow { .. } => Some(DockEvent::WindowOpened),
        HyprlandEvent::CloseWindow { .. } => Some(DockEvent::WindowClosed),
        HyprlandEvent::ActiveWindow { class, .. } => {
            Some(DockEvent::ActiveWindowChanged(Some(class.clone())))
        }
        HyprlandEvent::ActiveWindowV2 { address } => {
            let clients = hyprland.clients.get();
            let focused_app = clients.iter()
                .find(|c| c.address.get() == *address)
                .map(|c| c.class.get());
            Some(DockEvent::ActiveWindowChanged(focused_app))
        }
        HyprlandEvent::Minimized { .. } => {
            Some(DockEvent::WindowsChanged)
        }
        HyprlandEvent::MoveWindow { .. }
        | HyprlandEvent::MoveWindowV2 { .. } => Some(DockEvent::WindowsChanged),
        _ => None,
    }
}
