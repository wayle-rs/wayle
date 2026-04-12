//! Watches config properties that affect settings dialog CSS and
//! triggers a CSS reload when any of them change.

use std::{future::ready, sync::Arc};

use futures::StreamExt;
use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges, schemas::styling::ThemeProvider};

use crate::app::{SettingsApp, SettingsAppCmd};

pub fn spawn(sender: &ComponentSender<SettingsApp>, config_service: &Arc<ConfigService>) {
    let config = config_service.config();

    watch_property(sender, config.styling.scale.watch());
    watch_property(sender, config.styling.rounding.watch());
    watch_property(sender, config.general.font_sans.watch());

    watch_property(
        sender,
        config
            .styling
            .theme_provider
            .watch()
            .filter(|provider| ready(*provider == ThemeProvider::Wayle)),
    );

    watch_palette(sender, config_service);
}

fn watch_property<T: Send + 'static>(
    sender: &ComponentSender<SettingsApp>,
    stream: impl futures::Stream<Item = T> + Send + 'static,
) {
    sender.command(|out, shutdown| async move {
        let mut stream = Box::pin(stream);
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        stream.next().await;

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                item = stream.next() => {
                    if item.is_none() { break; }
                    let _ = out.send(SettingsAppCmd::CssReloadNeeded);
                }
            }
        }
    });
}

fn watch_palette(sender: &ComponentSender<SettingsApp>, config_service: &Arc<ConfigService>) {
    let config = config_service.config();
    let (tx, mut rx) = mpsc::unbounded_channel();
    config.styling.palette.subscribe_changes(tx);

    sender.command(|out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                msg = rx.recv() => {
                    if msg.is_none() { break; }
                    let _ = out.send(SettingsAppCmd::CssReloadNeeded);
                }
            }
        }
    });
}
