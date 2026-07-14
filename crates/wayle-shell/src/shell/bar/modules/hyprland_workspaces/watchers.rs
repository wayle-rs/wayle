use std::sync::Arc;

use futures::StreamExt;
use relm4::ComponentSender;
use tracing::warn;
use wayle_config::{
    ConfigProperty,
    schemas::{
        modules::HyprlandWorkspacesConfig,
        styling::{ScaleFactor, ThemeProvider},
    },
};
use wayle_hyprland::{HyprlandEvent, HyprlandService};
use wayle_widgets::{prelude::BarSettings, watch};

use super::HyprlandWorkspaces;
use crate::shell::bar::modules::hyprland_workspaces::messages::WorkspacesCmd;

pub(super) fn spawn_watchers(
    sender: &ComponentSender<HyprlandWorkspaces>,
    config: &HyprlandWorkspacesConfig,
    hyprland: &Option<Arc<HyprlandService>>,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    spawn_hyprland_watchers(sender, hyprland);
    spawn_config_watchers(sender, config, theme_provider, bar_scale, settings);
}

fn spawn_hyprland_watchers(
    sender: &ComponentSender<HyprlandWorkspaces>,
    hyprland: &Option<Arc<HyprlandService>>,
) {
    let Some(hyprland) = hyprland.clone() else {
        warn!(
            service = "HyprlandService",
            module = "hyprland-workspaces",
            "unavailable, skipping watcher"
        );
        return;
    };

    let workspaces = hyprland.workspaces.clone();
    watch!(sender, [workspaces.watch()], |out| {
        let _ = out.send(WorkspacesCmd::WorkspacesChanged);
    });

    let clients = hyprland.clients.clone();
    watch!(sender, [clients.watch()], |out| {
        let _ = out.send(WorkspacesCmd::ClientsChanged);
    });

    let hyprland_clone = hyprland.clone();
    sender.command(move |out, shutdown| watch_title_events(hyprland_clone.clone(), out, shutdown));

    sender.command({
        let hyprland_clone = hyprland.clone();
        move |out, shutdown| watch_workspace_events(hyprland_clone.clone(), out, shutdown)
    });
}

async fn watch_workspace_events(
    hyprland: Arc<HyprlandService>,
    out: relm4::Sender<WorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut events = hyprland.events();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            event = events.next() => {
                let Some(event) = event else { continue; };
                match event {
                    HyprlandEvent::WorkspaceV2 { id, .. } => {
                        let _ = out.send(WorkspacesCmd::ActiveWorkspaceChanged(id));
                    }
                    HyprlandEvent::FocusedMonV2 {
                        name,
                        workspace_id,
                    } => {
                        let _ = out.send(WorkspacesCmd::MonitorFocused {
                            monitor: name,
                            workspace_id,
                        });
                    }
                    HyprlandEvent::CreateWorkspaceV2 { .. }
                    | HyprlandEvent::DestroyWorkspaceV2 { .. }
                    | HyprlandEvent::MoveWorkspaceV2 { .. }
                    | HyprlandEvent::RenameWorkspace { .. }
                    | HyprlandEvent::ActiveSpecialV2 { .. }
                    | HyprlandEvent::MonitorAddedV2 { .. }
                    | HyprlandEvent::MonitorRemovedV2 { .. } => {
                        let _ = out.send(WorkspacesCmd::WorkspacesChanged);
                    }
                    HyprlandEvent::OpenWindow { .. }
                    | HyprlandEvent::CloseWindow { .. }
                    | HyprlandEvent::MoveWindow { .. }
                    | HyprlandEvent::MoveWindowV2 { .. } => {
                        let _ = out.send(WorkspacesCmd::ClientsChanged);
                    }
                    HyprlandEvent::Urgent { address } => {
                        let _ = out.send(WorkspacesCmd::UrgentWindow(address));
                    }
                    HyprlandEvent::ActiveWindowV2 { address } => {
                        let _ = out.send(WorkspacesCmd::WindowFocused(address));
                    }
                    HyprlandEvent::ConfigReloaded => {
                        let _ = out.send(WorkspacesCmd::HyprlandConfigReloaded);
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn watch_title_events(
    hyprland: Arc<HyprlandService>,
    out: relm4::Sender<WorkspacesCmd>,
    shutdown: relm4::ShutdownReceiver,
) {
    let mut events = hyprland.events();
    let shutdown_fut = shutdown.wait();
    tokio::pin!(shutdown_fut);

    loop {
        tokio::select! {
            () = &mut shutdown_fut => return,
            event = events.next() => {
                if let Some(HyprlandEvent::WindowTitleV2 { .. }) = event {
                    let _ = out.send(WorkspacesCmd::TitleChanged);
                }
            }
        }
    }
}

fn spawn_config_watchers(
    sender: &ComponentSender<HyprlandWorkspaces>,
    config: &HyprlandWorkspacesConfig,
    theme_provider: ConfigProperty<ThemeProvider>,
    bar_scale: ConfigProperty<ScaleFactor>,
    settings: &BarSettings,
) {
    let min_count = config.min_workspace_count.clone();
    let monitor_specific = config.monitor_specific.clone();
    let show_special = config.show_special.clone();
    let urgent_show = config.urgent_show.clone();
    let display_mode = config.display_mode.clone();
    let label_use_name = config.label_use_name.clone();
    let numbering = config.numbering.clone();
    let divider = config.divider.clone();
    let app_icons_show = config.app_icons_show.clone();
    let app_icons_dedupe = config.app_icons_dedupe.clone();
    let app_icons_fallback = config.app_icons_fallback.clone();
    let app_icons_empty = config.app_icons_empty.clone();
    let icon_gap = config.icon_gap.clone();
    let icon_size = config.icon_size.clone();
    let label_size = config.label_size.clone();
    let workspace_padding = config.workspace_padding.clone();
    let workspace_ignore = config.workspace_ignore.clone();
    let active_indicator = config.active_indicator.clone();
    let active_color = config.active_color.clone();
    let occupied_color = config.occupied_color.clone();
    let empty_color = config.empty_color.clone();
    let container_bg_color = config.container_bg_color.clone();
    let border_show = config.border_show.clone();
    let border_color = config.border_color.clone();
    let workspace_map = config.workspace_map.clone();
    let app_icon_map = config.app_icon_map.clone();
    let border_width = settings.border_width.clone();
    let border_location = settings.border_location.clone();
    let is_vertical = settings.is_vertical.clone();

    watch!(
        sender,
        [
            min_count.watch(),
            monitor_specific.watch(),
            show_special.watch(),
            urgent_show.watch(),
            display_mode.watch(),
            label_use_name.watch(),
            numbering.watch(),
            divider.watch(),
            app_icons_show.watch(),
            app_icons_dedupe.watch(),
            app_icons_fallback.watch(),
            app_icons_empty.watch(),
            icon_gap.watch(),
            icon_size.watch(),
            label_size.watch(),
            workspace_padding.watch(),
            workspace_ignore.watch(),
            active_indicator.watch(),
            active_color.watch(),
            occupied_color.watch(),
            empty_color.watch(),
            container_bg_color.watch(),
            border_show.watch(),
            border_color.watch(),
            workspace_map.watch(),
            app_icon_map.watch(),
            theme_provider.watch(),
            bar_scale.watch(),
            border_width.watch(),
            border_location.watch(),
            is_vertical.watch()
        ],
        |out| {
            let _ = out.send(WorkspacesCmd::ConfigChanged);
        }
    );
}
