use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use wayle_config::{ConfigProperty, ConfigService};
use wayle_systray::SystemTrayService;
use wayle_widgets::watch;

use super::{SystrayCmd, SystrayModule};
use crate::services::shell_ipc::ShellIpcState;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<SystrayModule>,
    is_vertical: &ConfigProperty<bool>,
    systray: &Arc<SystemTrayService>,
    config_service: &Arc<ConfigService>,
    shell_ipc: &ShellIpcState,
    connector: Option<String>,
) {
    let full_config = config_service.config();
    let systray_config = &full_config.modules.systray;
    let bar_config = &full_config.bar;

    let items = systray.items.clone();
    let blacklist = systray_config.blacklist.clone();
    let overrides = systray_config.overrides.clone();
    watch!(
        sender,
        [items.watch(), blacklist.watch(), overrides.watch()],
        |out| {
            let _ = out.send(SystrayCmd::ItemsChanged(items.get()));
        }
    );

    let item_gap = systray_config.item_gap.clone();
    let icon_scale = systray_config.icon_scale.clone();
    let internal_padding = systray_config.internal_padding.clone();
    let bar_scale = bar_config.scale.clone();
    watch!(
        sender,
        [
            item_gap.watch(),
            icon_scale.watch(),
            internal_padding.watch(),
            bar_scale.watch()
        ],
        |out| {
            let _ = out.send(SystrayCmd::StylingChanged);
        }
    );

    let is_vertical = is_vertical.clone();
    watch!(sender, [is_vertical.watch()], |out| {
        let _ = out.send(SystrayCmd::OrientationChanged(is_vertical.get()));
    });

    // `wayle systray toggle`/`open <id>` requests, drained from the bounded log via a
    // per-bar cursor: start caught up to the current log (so a bar created after a
    // request doesn't replay it), then act once on every newer entry aimed at this bar
    // (or all bars). A burst arrives as one log, so none is coalesced away.
    let systray_open = shell_ipc.systray_menu_request.clone();
    let mut request_stream = systray_open.watch();
    let mut cursor = systray_open.get().last().map_or(0, |r| r.nonce);
    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,

                Some(log) = request_stream.next() => {
                    for request in log {
                        if request.nonce <= cursor {
                            continue;
                        }
                        cursor = request.nonce;
                        let for_this_bar = request.monitor.is_empty()
                            || connector.as_deref() == Some(request.monitor.as_str());
                        if for_this_bar {
                            let _ = out.send(SystrayCmd::MenuRequest(request.action, request.id));
                        }
                    }
                }
            }
        }
    });
}
