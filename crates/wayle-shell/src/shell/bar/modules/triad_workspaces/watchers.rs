//! Background watchers: triad event stream + config-property changes.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{
    ConfigProperty, SubscribeChanges,
    schemas::{
        modules::TriadWorkspacesConfig,
        styling::{ScaleFactor, ThemeProvider},
    },
};
use wayle_triad::{TriadEvent, TriadService};
use wayle_widgets::prelude::BarSettings;

use super::{TriadWorkspaces, messages::TriadWorkspacesCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<TriadWorkspaces>,
    config: &TriadWorkspacesConfig,
    triad: Arc<TriadService>,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    spawn_triad_events(sender, triad);
    spawn_config_watcher(sender, config, theme_provider, bar_scale, settings);
}

fn spawn_triad_events(sender: &ComponentSender<TriadWorkspaces>, triad: Arc<TriadService>) {
    sender.command(move |out, shutdown| watch_workspace_events(triad.clone(), out, shutdown));
}

async fn watch_workspace_events(
    triad: Arc<TriadService>,
    out: relm4::Sender<TriadWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut events = triad.events();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            event = events.next() => {
                let Some(event) = event else { return };
                let cmd = event_to_cmd(event);
                let _ = out.send(cmd);
            }
        }
    }
}

fn event_to_cmd(event: TriadEvent) -> TriadWorkspacesCmd {
    match event {
        TriadEvent::StateChanged
        | TriadEvent::LayoutStateChanged
        | TriadEvent::WindowChanged { .. } => TriadWorkspacesCmd::WorkspacesChanged,
    }
}

fn spawn_config_watcher(
    sender: &ComponentSender<TriadWorkspaces>,
    config: &TriadWorkspacesConfig,
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
    out: relm4::Sender<TriadWorkspacesCmd>,
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

                let _ = out.send(TriadWorkspacesCmd::ConfigChanged);
            }
        }
    }
}
