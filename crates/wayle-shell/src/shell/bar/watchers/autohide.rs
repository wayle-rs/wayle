use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges};

use crate::shell::bar::{Bar, BarCmd};

pub(crate) fn spawn(sender: &ComponentSender<Bar>, config_service: &Arc<ConfigService>) {
    let autohide_prop = config_service.config().bar.autohide.clone();
    let timeout_prop = config_service.config().bar.autohide_timeout.clone();
    let trigger_size_prop = config_service.config().bar.autohide_trigger_size.clone();

    let (tx_auto, mut rx_auto) = mpsc::unbounded_channel();
    autohide_prop.subscribe_changes(tx_auto);

    let (tx_timeout, mut rx_timeout) = mpsc::unbounded_channel();
    timeout_prop.subscribe_changes(tx_timeout);

    let (tx_size, mut rx_size) = mpsc::unbounded_channel();
    trigger_size_prop.subscribe_changes(tx_size);

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(()) = rx_auto.recv() => {
                    let _ = out.send(BarCmd::AutohideChanged(autohide_prop.get()));
                }
                Some(()) = rx_timeout.recv() => {
                    let _ = out.send(BarCmd::AutohideTimeoutChanged(timeout_prop.get()));
                }
                Some(()) = rx_size.recv() => {
                    let _ = out.send(BarCmd::AutohideTriggerSizeChanged(trigger_size_prop.get()));
                }
            }
        }
    });
}
