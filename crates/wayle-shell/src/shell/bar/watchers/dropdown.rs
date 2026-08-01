//! Watches for CLI `dropdown` requests (`toggle`/`open`/`close`) and forwards
//! matching ones to this bar. The daemon validates the request and publishes it on
//! the reactive `dropdown_request`; each bar's watcher fires when the request
//! targets its connector (or all bars).
//!
//! Also republishes this bar's dropdown identifiers on every config reload, so
//! `wayle dropdown list` and CLI addressing track click bindings re-configured at
//! runtime (the openers read their names live; this just re-runs the rebuild).

use std::sync::Arc;

use futures::StreamExt;
use relm4::{
    ComponentSender,
    gtk::{gdk, prelude::*},
};
use wayle_config::ConfigService;

use crate::{
    services::shell_ipc::ShellIpcState,
    shell::bar::{Bar, BarCmd},
};

pub(crate) fn spawn(
    sender: &ComponentSender<Bar>,
    monitor: &gdk::Monitor,
    config_service: &Arc<ConfigService>,
    ipc_state: &ShellIpcState,
) {
    let ipc = ipc_state.clone();
    let connector = monitor
        .connector()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut request_stream = ipc.dropdown_request.watch();
    // Start caught up to the current log, so a bar (re)created after requests were
    // issued doesn't replay them; then act once on every entry newer than this cursor.
    let mut cursor = ipc.dropdown_request.get().last().map_or(0, |r| r.nonce);

    sender.command(move |out, shutdown| async move {
        let shutdown_fut = shutdown.wait();
        tokio::pin!(shutdown_fut);

        loop {
            tokio::select! {
                () = &mut shutdown_fut => break,

                Some(log) = request_stream.next() => {
                    // Drain every request newer than the cursor (a burst arrives as one
                    // log, so none is coalesced away); advance the cursor for all of
                    // them, but only dispatch those targeting this bar.
                    for request in log {
                        if request.nonce <= cursor {
                            continue;
                        }
                        cursor = request.nonce;
                        if request.monitor.is_empty() || request.monitor == connector {
                            let _ = out.send(BarCmd::Dropdown(request.action, request.identifier));
                        }
                    }
                }
            }
        }
    });

    // Re-derive this bar's dropdown identifiers whenever the config is reloaded, so a
    // click binding re-pointed at a different dropdown is reflected in `dropdown list`
    // and CLI addressing. One coarse signal (per debounced reload) instead of watching
    // every module's click properties individually.
    if let Some(mut reload_rx) = config_service.subscribe_config_reload() {
        sender.command(move |out, shutdown| async move {
            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            loop {
                tokio::select! {
                    () = &mut shutdown_fut => break,

                    result = reload_rx.changed() => {
                        if result.is_err() {
                            break;
                        }
                        let _ = out.send(BarCmd::RepublishDropdowns);
                    }
                }
            }
        });
    }
}
