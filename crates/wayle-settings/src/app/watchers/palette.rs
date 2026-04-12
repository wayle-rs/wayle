//! Watches the wallpaper service's `ColorsExtracted` D-Bus signal
//! and rebuilds CSS when color extraction completes.

use futures::StreamExt;
use relm4::ComponentSender;
use tracing::{info, warn};
use wayle_wallpaper::WallpaperProxy;

use crate::app::{SettingsApp, SettingsAppCmd};

pub(crate) fn spawn(sender: &ComponentSender<SettingsApp>) {
    sender.command(|out, shutdown| async move {
        let Ok(connection) = zbus::Connection::session().await else {
            warn!("no D-Bus session, palette watcher disabled");
            return;
        };

        let proxy = match WallpaperProxy::new(&connection).await {
            Ok(proxy) => proxy,
            Err(err) => {
                warn!(error = %err, "wallpaper proxy unavailable, palette watcher disabled");
                return;
            }
        };

        let Ok(mut signal_stream) = proxy.receive_colors_extracted().await else {
            warn!("cannot subscribe to ColorsExtracted, palette watcher disabled");
            return;
        };

        info!("palette watcher connected");

        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                signal = signal_stream.next() => {
                    if signal.is_none() { break; }
                    info!("ColorsExtracted signal received");
                    let _ = out.send(SettingsAppCmd::CssReloadNeeded);
                }
            }
        }
    });
}
