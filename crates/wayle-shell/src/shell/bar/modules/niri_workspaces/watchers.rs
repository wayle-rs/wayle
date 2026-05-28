//! Background watchers: niri event stream + config-property changes.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{
    ConfigProperty, SubscribeChanges,
    schemas::{
        modules::NiriWorkspacesConfig,
        styling::{ScaleFactor, ThemeProvider},
    },
};
use wayle_niri::{Event, NiriService};
use wayle_widgets::prelude::BarSettings;

use super::{NiriWorkspaces, messages::NiriWorkspacesCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<NiriWorkspaces>,
    config: &NiriWorkspacesConfig,
    niri: Arc<NiriService>,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    spawn_niri_events(sender, niri);
    spawn_config_watcher(sender, config, theme_provider, bar_scale, settings);
}

fn spawn_niri_events(sender: &ComponentSender<NiriWorkspaces>, niri: Arc<NiriService>) {
    sender.command(move |out, shutdown| watch_workspace_events(niri.clone(), out, shutdown));
}

async fn watch_workspace_events(
    niri: Arc<NiriService>,
    out: relm4::Sender<NiriWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut events = niri.events();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            event = events.next() => {
                let Some(event) = event else { return };
                let Some(cmd) = event_to_cmd(event) else { continue };
                let _ = out.send(cmd);
            }
        }
    }
}

fn event_to_cmd(event: Event) -> Option<NiriWorkspacesCmd> {
    match event {
        Event::WorkspacesChanged { .. }
        | Event::WorkspaceUrgencyChanged { .. }
        | Event::WorkspaceActivated { .. }
        | Event::WorkspaceActiveWindowChanged { .. }
        | Event::WindowsChanged { .. }
        | Event::WindowOpenedOrChanged { .. }
        | Event::WindowClosed { .. }
        | Event::WindowFocusChanged { .. }
        | Event::WindowLayoutsChanged { .. }
        | Event::WindowUrgencyChanged { .. } => Some(NiriWorkspacesCmd::WorkspacesChanged),
        _ => None,
    }
}

fn spawn_config_watcher(
    sender: &ComponentSender<NiriWorkspaces>,
    config: &NiriWorkspacesConfig,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    let (tx, rx) = mpsc::unbounded_channel();

    config.subscribe_changes(tx.clone());
    theme_provider.subscribe_changes(tx.clone());
    bar_scale.subscribe_changes(tx.clone());
    settings.border_width.subscribe_changes(tx.clone());
    settings.border_location.subscribe_changes(tx.clone());
    settings.is_vertical.subscribe_changes(tx);

    sender.command(move |out, shutdown| watch_config_changes(rx, out, shutdown));
}

async fn watch_config_changes(
    mut rx: mpsc::UnboundedReceiver<()>,
    out: relm4::Sender<NiriWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            received = rx.recv() => {
                if received.is_none() {
                    return;
                }

                while rx.try_recv().is_ok() {}

                let _ = out.send(NiriWorkspacesCmd::ConfigChanged);
            }
        }
    }
}
