//! Background watchers: service Property changes + config-property changes.

use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{
    ConfigProperty, SubscribeChanges,
    schemas::{modules::ExtWorkspacesConfig, styling::ThemeProvider},
};
use wayle_ext_workspace::ExtWorkspaceService;
use wayle_widgets::{prelude::BarSettings, watch};

use super::{ExtWorkspaces, ExtWorkspacesCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<ExtWorkspaces>,
    config: &ExtWorkspacesConfig,
    service: &Arc<ExtWorkspaceService>,
    theme_provider: ConfigProperty<ThemeProvider>,
    settings: &BarSettings,
) {
    spawn_service_watcher(sender, service);
    spawn_config_watcher(sender, config, theme_provider, settings);
}

fn spawn_service_watcher(
    sender: &ComponentSender<ExtWorkspaces>,
    service: &Arc<ExtWorkspaceService>,
) {
    let workspaces = service.workspaces.clone();
    let groups = service.groups.clone();
    watch!(sender, [workspaces.watch(), groups.watch()], |out| {
        let _ = out.send(ExtWorkspacesCmd::WorkspacesChanged);
    });
}

fn spawn_config_watcher(
    sender: &ComponentSender<ExtWorkspaces>,
    config: &ExtWorkspacesConfig,
    theme_provider: ConfigProperty<ThemeProvider>,
    settings: &BarSettings,
) {
    let (tx, rx) = mpsc::unbounded_channel();

    config.subscribe_changes(tx.clone());
    theme_provider.subscribe_changes(tx.clone());
    settings.border_width.subscribe_changes(tx.clone());
    settings.border_location.subscribe_changes(tx.clone());
    settings.is_vertical.subscribe_changes(tx);

    sender.command(move |out, shutdown| watch_config_changes(rx, out, shutdown));
}

async fn watch_config_changes(
    mut rx: mpsc::UnboundedReceiver<()>,
    out: relm4::Sender<ExtWorkspacesCmd>,
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

                let _ = out.send(ExtWorkspacesCmd::ConfigChanged);
            }
        }
    }
}
