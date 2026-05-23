//! Background watchers spawned during component init: layout-stream
//! consumer plus config-property change subscriptions.

use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use wayle_config::schemas::modules::KeyboardInputConfig;
use wayle_widgets::watch;

use super::{component::KeyboardInput, messages::KeyboardInputCmd, sources::KeyboardLayoutSource};

pub(super) fn spawn_watchers(
    sender: &ComponentSender<KeyboardInput>,
    config: &KeyboardInputConfig,
    source: Arc<dyn KeyboardLayoutSource>,
) {
    spawn_layout_watcher(sender, source);
    spawn_config_watchers(sender, config);
}

fn spawn_layout_watcher(
    sender: &ComponentSender<KeyboardInput>,
    source: Arc<dyn KeyboardLayoutSource>,
) {
    sender.command(move |out, shutdown| watch_layout_changes(source, out, shutdown));
}

async fn watch_layout_changes(
    source: Arc<dyn KeyboardLayoutSource>,
    out: relm4::Sender<KeyboardInputCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut changes = source.changes();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            next = changes.next() => {
                let Some(layout) = next else { return };
                let _ = out.send(KeyboardInputCmd::LayoutChanged(layout));
            }
        }
    }
}

fn spawn_config_watchers(sender: &ComponentSender<KeyboardInput>, config: &KeyboardInputConfig) {
    let format = config.format.clone();
    watch!(sender, [format.watch()], |out| {
        let _ = out.send(KeyboardInputCmd::FormatChanged);
    });

    let icon_name = config.icon_name.clone();
    watch!(sender, [icon_name.watch()], |out| {
        let _ = out.send(KeyboardInputCmd::UpdateIcon(icon_name.get().clone()));
    });

    let layout_alias_map = config.layout_alias_map.clone();
    watch!(sender, [layout_alias_map.watch()], |out| {
        let _ = out.send(KeyboardInputCmd::LayoutAliasMapChanged);
    });
}
