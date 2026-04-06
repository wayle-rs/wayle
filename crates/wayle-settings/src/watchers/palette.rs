//! Watches the wallpaper service's `ColorsExtracted` D-Bus signal
//! and notifies the app to rebuild CSS with the new palette.

use futures::StreamExt;
use relm4::ComponentSender;
use tracing::debug;

use crate::app::{SettingsApp, SettingsAppCmd};

pub fn spawn_palette_watcher(sender: &ComponentSender<SettingsApp>) {
    sender.command(|out, shutdown| async move {
        let Ok(connection) = zbus::Connection::session().await else {
            debug!("no D-Bus session, palette signal unavailable");
            return;
        };

        let proxy = match wayle_wallpaper::WallpaperProxy::new(&connection).await {
            Ok(proxy) => proxy,
            Err(err) => {
                debug!(error = %err, "wallpaper proxy unavailable, no reactive palette");
                return;
            }
        };

        let Ok(mut signal_stream) = proxy.receive_colors_extracted().await else {
            debug!("cannot subscribe to colors_extracted signal");
            return;
        };

        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                signal = signal_stream.next() => {
                    if signal.is_none() { break; }
                    let _ = out.send(SettingsAppCmd::CssReloadNeeded);
                }
            }
        }
    });
}
