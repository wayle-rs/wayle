use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges};

use crate::shell::bar::{Bar, BarCmd};

pub(crate) fn spawn(sender: &ComponentSender<Bar>, config_service: &Arc<ConfigService>) {
    let config = config_service.config();
    let layer_prop = config.bar.layer.clone();
    let tearing_prop = config.general.tearing_mode.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    layer_prop.subscribe_changes(tx.clone());
    tearing_prop.subscribe_changes(tx);

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(()) = rx.recv() => {
                    let _ = out.send(BarCmd::LayerChanged);
                }
            }
        }
    });
}
