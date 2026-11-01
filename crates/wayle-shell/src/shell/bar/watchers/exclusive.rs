use std::sync::Arc;

use relm4::ComponentSender;
use tokio::sync::mpsc;
use wayle_config::{ConfigService, SubscribeChanges};

use crate::shell::bar::{Bar, BarCmd};

pub(crate) fn spawn(sender: &ComponentSender<Bar>, config_service: &Arc<ConfigService>) {
    let exclusive_prop = config_service.config().bar.exclusive.clone();

    let (tx, mut rx) = mpsc::unbounded_channel();
    exclusive_prop.subscribe_changes(tx);

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,
                Some(()) = rx.recv() => {
                    let _ = out.send(BarCmd::ExclusiveChanged(exclusive_prop.get()));
                }
            }
        }
    });
}
