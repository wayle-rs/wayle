//! Background watchers spawned during component init: focus-stream
//! consumer plus config-property change subscriptions.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use wayle_config::{ConfigProperty, schemas::modules::WindowTitleConfig};
use wayle_widgets::watch;

use super::{component::WindowTitle, messages::WindowTitleCmd, sources::FocusedWindowSource};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<WindowTitle>,
    config: &WindowTitleConfig,
    source: Arc<dyn FocusedWindowSource>,
) {
    spawn_window_watcher(sender, config, source);
    spawn_config_watchers(sender, config);
}

fn spawn_window_watcher(
    sender: &ComponentSender<WindowTitle>,
    config: &WindowTitleConfig,
    source: Arc<dyn FocusedWindowSource>,
) {
    let format = config.format.clone();
    sender.command(move |out, shutdown| watch_window_changes(source, format, out, shutdown));
}

async fn watch_window_changes(
    source: Arc<dyn FocusedWindowSource>,
    format: ConfigProperty<String>,
    out: relm4::Sender<WindowTitleCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut changes = source.changes();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            next = changes.next() => {
                let Some(focused) = next else { return };
                let _ = out.send(WindowTitleCmd::WindowChanged {
                    focused,
                    format: format.get(),
                });
            }
        }
    }
}

fn spawn_config_watchers(sender: &ComponentSender<WindowTitle>, config: &WindowTitleConfig) {
    let format = config.format.clone();
    watch!(sender, [format.watch()], |out| {
        let _ = out.send(WindowTitleCmd::FormatChanged);
    });

    let icon_name = config.icon_name.clone();
    let icon_mappings = config.icon_mappings.clone();
    watch!(sender, [icon_name.watch(), icon_mappings.watch()], |out| {
        let _ = out.send(WindowTitleCmd::IconConfigChanged);
    });
}
