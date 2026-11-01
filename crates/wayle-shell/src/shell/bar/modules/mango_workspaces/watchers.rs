//! Background watchers: mango monitor and client streams + config-property
//! changes.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{
    ConfigProperty, SubscribeChanges,
    schemas::{
        modules::MangoWorkspacesConfig,
        styling::{ScaleFactor, ThemeProvider},
    },
};
use wayle_mango::MangoService;
use wayle_widgets::prelude::BarSettings;

use super::{MangoWorkspaces, messages::MangoWorkspacesCmd};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<MangoWorkspaces>,
    config: &MangoWorkspacesConfig,
    mango: Arc<MangoService>,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    spawn_service_watcher(sender, mango);
    spawn_config_watcher(sender, config, theme_provider, bar_scale, settings);
}

fn spawn_service_watcher(sender: &ComponentSender<MangoWorkspaces>, mango: Arc<MangoService>) {
    sender.command(move |out, shutdown| watch_service_changes(mango.clone(), out, shutdown));
}

async fn watch_service_changes(
    mango: Arc<MangoService>,
    out: relm4::Sender<MangoWorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut monitors = mango.monitors.watch();
    let mut clients = mango.clients.watch();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            change = monitors.next() => {
                if change.is_none() {
                    return;
                }
                let _ = out.send(MangoWorkspacesCmd::TagsChanged);
            }
            change = clients.next() => {
                if change.is_none() {
                    return;
                }
                let _ = out.send(MangoWorkspacesCmd::TagsChanged);
            }
        }
    }
}

fn spawn_config_watcher(
    sender: &ComponentSender<MangoWorkspaces>,
    config: &MangoWorkspacesConfig,
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
    out: relm4::Sender<MangoWorkspacesCmd>,
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

                let _ = out.send(MangoWorkspacesCmd::ConfigChanged);
            }
        }
    }
}
