//! [`MangoWorkspaces`] private impl methods: rebuild, border classes,
//! blink timer, and action dispatch.

use std::collections::{HashMap, HashSet};

use gtk::prelude::*;
use relm4::{ComponentSender, gtk};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_config::{
    ClickAction,
    schemas::{
        bar::BorderLocation,
        modules::{ActiveIndicator, DisplayMode, UrgentMode, WorkspaceClickAction, WorkspaceStyle},
    },
};
use wayle_mango::{Client, Monitor, Tag, TagId};

use super::{
    BLINK_INTERVAL, MangoWorkspaces, MangoWorkspacesCmd,
    button::{AppIconInit, MangoTagButtonInit},
    helpers,
};
use crate::{process, shell::bar::dropdowns};

const REM_BASE_PX: f32 = 16.0;

struct TagLayout {
    display_mode: DisplayMode,
    active_indicator: ActiveIndicator,
    urgent_show: bool,
    urgent_mode: UrgentMode,
    is_vertical: bool,
    divider: String,
    app_icons_show: bool,
    app_icons_dedupe: bool,
    app_icons_fallback: String,
    app_icons_empty: String,
    icon_gap_px: i32,
    app_icon_map: HashMap<String, String>,
    tag_map: HashMap<String, WorkspaceStyle>,
    blink_on: bool,
}

impl MangoWorkspaces {
    pub(super) fn bar_monitor(&self) -> Option<&str> {
        self.settings.monitor_name.as_deref()
    }

    pub(super) fn is_vertical(&self) -> bool {
        self.settings.is_vertical.get()
    }

    pub(super) fn orientation(&self) -> gtk::Orientation {
        if self.is_vertical() {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(super) fn rebuild_tags(&mut self) {
        let config = self.config.config();
        let tags_config = &config.modules.mango_workspaces;

        let hide_empty = tags_config.hide_empty.get();
        let min_tag_count = tags_config.min_tag_count.get();
        let urgent_show = tags_config.urgent_show.get();
        let border_show = tags_config.border_show.get();

        let layout = TagLayout {
            display_mode: tags_config.display_mode.get(),
            active_indicator: tags_config.active_indicator.get(),
            urgent_show,
            urgent_mode: tags_config.urgent_mode.get(),
            is_vertical: self.is_vertical(),
            divider: tags_config.divider.get(),
            app_icons_show: tags_config.app_icons_show.get(),
            app_icons_dedupe: tags_config.app_icons_dedupe.get(),
            app_icons_fallback: tags_config.app_icons_fallback.get(),
            app_icons_empty: tags_config.app_icons_empty.get(),
            icon_gap_px: (tags_config.icon_gap.get().value() * REM_BASE_PX).round() as i32,
            app_icon_map: tags_config.app_icon_map.get(),
            tag_map: tags_config.tag_map.get(),
            blink_on: self.blink_on,
        };

        let monitor = self.chosen_monitor();
        let displayed = monitor
            .as_ref()
            .map(|monitor| displayed_tags(monitor, hide_empty, min_tag_count))
            .unwrap_or_default();
        let clients = monitor
            .as_ref()
            .map(|monitor| self.scoped_clients(&monitor.name))
            .unwrap_or_default();

        self.urgent_present = urgent_show && any_urgent(&displayed, &clients);

        self.repopulate(&displayed, &clients, &layout);
        self.update_border_classes(border_show);
    }

    /// The monitor whose tags this bar shows.
    ///
    /// Mango tags are per-monitor, so a bar only ever shows its own monitor's
    /// tags. Falls back to the active or first monitor if the bar's connector
    /// is not present in the current snapshot.
    fn chosen_monitor(&self) -> Option<Monitor> {
        let monitors = self.mango.monitors.get();

        if let Some(name) = self.bar_monitor()
            && let Some(found) = monitors.iter().find(|monitor| monitor.name == name)
        {
            return Some(found.clone());
        }

        monitors
            .iter()
            .find(|monitor| monitor.is_active)
            .or_else(|| monitors.first())
            .cloned()
    }

    fn scoped_clients(&self, monitor_name: &str) -> Vec<Client> {
        clients_for_monitor(self.mango.clients.get(), monitor_name)
    }

    fn repopulate(&mut self, displayed: &[Tag], clients: &[Client], layout: &TagLayout) {
        let mut guard = self.buttons.guard();
        guard.clear();

        for tag in displayed {
            guard.push_back(build_button_init(tag, clients, layout));
        }
    }

    pub(super) fn sync_blink(&mut self, sender: &ComponentSender<MangoWorkspaces>) {
        match (self.urgent_present, self.blink_token.is_some()) {
            (true, false) => self.start_blink_timer(sender),
            (false, true) => self.stop_blink_timer(),
            _ => {}
        }
    }

    fn start_blink_timer(&mut self, sender: &ComponentSender<MangoWorkspaces>) {
        self.stop_blink_timer();
        self.blink_on = true;

        let token = CancellationToken::new();
        let cancel = token.clone();
        self.blink_token = Some(token);

        sender.command(move |out, shutdown| async move {
            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            let mut interval = tokio::time::interval(BLINK_INTERVAL);
            interval.tick().await;

            loop {
                tokio::select! {
                    () = &mut shutdown_fut => break,
                    () = cancel.cancelled() => break,
                    _ = interval.tick() => {
                        let _ = out.send(MangoWorkspacesCmd::BlinkTick);
                    }
                }
            }
        });
    }

    fn stop_blink_timer(&mut self) {
        if let Some(token) = self.blink_token.take() {
            token.cancel();
        }
        self.blink_on = false;
    }

    fn update_border_classes(&self, show_border: bool) {
        let container = self.buttons.widget();

        for location in [
            BorderLocation::Top,
            BorderLocation::Bottom,
            BorderLocation::Left,
            BorderLocation::Right,
            BorderLocation::All,
        ] {
            if let Some(class) = location.css_class() {
                container.remove_css_class(class);
            }
        }

        if show_border && let Some(class) = self.settings.border_location.get().css_class() {
            container.add_css_class(class);
        }
    }

    pub(super) fn dispatch_click_action(&self, action: WorkspaceClickAction, clicked_index: u32) {
        match action {
            WorkspaceClickAction::None => {}
            WorkspaceClickAction::FocusWorkspace => self.spawn_view_tag(clicked_index),
            WorkspaceClickAction::FocusNext => self.spawn_view_right(),
            WorkspaceClickAction::FocusPrevious => self.spawn_view_left(),
            WorkspaceClickAction::FocusLast => {
                debug!("mango has no last-tag action; ignoring focus:last");
            }
            WorkspaceClickAction::Dropdown(name) => self.open_dropdown(name),
            WorkspaceClickAction::Shell(cmd) => process::run_if_set(&cmd),
        }
    }

    pub(super) fn dispatch_scroll_action(&self, action: WorkspaceClickAction) {
        match action {
            WorkspaceClickAction::None => {}
            WorkspaceClickAction::FocusWorkspace => {
                warn!("focus:this needs a clicked tag; scroll ignored");
            }
            WorkspaceClickAction::FocusNext => self.spawn_view_right(),
            WorkspaceClickAction::FocusPrevious => self.spawn_view_left(),
            WorkspaceClickAction::FocusLast => {
                debug!("mango has no last-tag action; ignoring focus:last");
            }
            WorkspaceClickAction::Dropdown(name) => self.open_dropdown(name),
            WorkspaceClickAction::Shell(cmd) => process::run_if_set(&cmd),
        }
    }

    fn open_dropdown(&self, name: String) {
        let action = ClickAction::Dropdown(name);
        dropdowns::dispatch_click_widget(&action, &self.dropdowns, self.buttons.widget());
    }

    fn spawn_view_tag(&self, index: u32) {
        let mango = self.mango.clone();
        tokio::spawn(async move {
            if let Err(err) = mango.view_tag(TagId::new(index)).await {
                warn!(error = %err, tag = index, "mango view_tag failed");
            }
        });
    }

    fn spawn_view_left(&self) {
        let mango = self.mango.clone();
        tokio::spawn(async move {
            if let Err(err) = mango.view_left().await {
                warn!(error = %err, "mango view_left failed");
            }
        });
    }

    fn spawn_view_right(&self) {
        let mango = self.mango.clone();
        tokio::spawn(async move {
            if let Err(err) = mango.view_right().await {
                warn!(error = %err, "mango view_right failed");
            }
        });
    }
}

fn displayed_tags(monitor: &Monitor, hide_empty: bool, min_tag_count: u8) -> Vec<Tag> {
    let min_index = u32::from(min_tag_count);

    monitor
        .tags
        .iter()
        .filter(|tag| {
            !hide_empty || tag.client_count > 0 || tag.is_active || tag.index.get() <= min_index
        })
        .cloned()
        .collect()
}

fn clients_for_monitor(clients: Vec<Client>, monitor_name: &str) -> Vec<Client> {
    clients
        .into_iter()
        .filter(|client| client.monitor == monitor_name)
        .collect()
}

fn any_urgent(displayed: &[Tag], clients: &[Client]) -> bool {
    let displayed_indices: HashSet<TagId> = displayed.iter().map(|tag| tag.index).collect();

    let urgent_tag = displayed.iter().any(|tag| tag.is_urgent);
    let urgent_client = clients.iter().any(|client| {
        client.is_urgent
            && client
                .tags
                .iter()
                .any(|index| displayed_indices.contains(index))
    });

    urgent_tag || urgent_client
}

fn clients_on_tag(clients: &[Client], index: TagId) -> Vec<&Client> {
    clients
        .iter()
        .filter(|client| client.tags.contains(&index))
        .collect()
}

fn collect_urgent_client_ids(clients: &[&Client]) -> HashSet<u32> {
    clients
        .iter()
        .filter(|client| client.is_urgent)
        .map(|client| client.id.get())
        .collect()
}

fn collect_app_icons(
    clients: &[&Client],
    app_icon_map: &HashMap<String, String>,
    fallback: &str,
    dedupe: bool,
) -> Vec<AppIconInit> {
    let mut result: Vec<AppIconInit> = Vec::with_capacity(clients.len());

    for client in clients {
        let icon_name = helpers::resolve_app_icon(
            client.app_id.as_deref(),
            client.title.as_deref(),
            app_icon_map,
            fallback,
        );

        if dedupe && let Some(existing) = result.iter_mut().find(|init| init.icon_name == icon_name)
        {
            existing.client_ids.push(client.id.get());
            continue;
        }

        result.push(AppIconInit {
            icon_name,
            client_ids: vec![client.id.get()],
        });
    }

    result
}

fn build_button_init(tag: &Tag, clients: &[Client], layout: &TagLayout) -> MangoTagButtonInit {
    let tag_clients = clients_on_tag(clients, tag.index);

    let urgent_client_ids = if layout.blink_on {
        collect_urgent_client_ids(&tag_clients)
    } else {
        HashSet::new()
    };

    let app_icons = if layout.app_icons_show {
        collect_app_icons(
            &tag_clients,
            &layout.app_icon_map,
            &layout.app_icons_fallback,
            layout.app_icons_dedupe,
        )
    } else {
        Vec::new()
    };

    let style = helpers::tag_style(tag.index.get(), &layout.tag_map);
    let icon = style.and_then(|style| style.icon.clone());
    let label = style
        .and_then(|style| style.label.clone())
        .unwrap_or_else(|| tag.index.to_string());

    MangoTagButtonInit {
        index: tag.index.get(),
        label: Some(label),
        icon,
        is_active: tag.is_active,
        is_urgent: tag.is_urgent && layout.blink_on,
        has_clients: tag.client_count > 0,
        is_vertical: layout.is_vertical,
        display_mode: layout.display_mode,
        active_indicator: layout.active_indicator,
        urgent_show: layout.urgent_show,
        urgent_mode: layout.urgent_mode,
        show_app_icons: layout.app_icons_show,
        app_icons,
        urgent_client_ids,
        divider: layout.divider.clone(),
        icon_gap_px: layout.icon_gap_px,
        empty_icon: layout.app_icons_empty.clone(),
    }
}

#[cfg(test)]
mod tests {
    use wayle_mango::ClientId;

    use super::*;

    fn tag(index: u32, is_active: bool, client_count: u32) -> Tag {
        Tag {
            index: TagId::new(index),
            is_active,
            is_urgent: false,
            layout: String::from("DW"),
            client_count,
        }
    }

    fn nine_tags(active: u32, occupied: u32) -> Vec<Tag> {
        (1..=9)
            .map(|index| tag(index, index == active, u32::from(index == occupied)))
            .collect()
    }

    fn monitor(name: &str, tags: Vec<Tag>) -> Monitor {
        Monitor {
            name: name.to_owned(),
            is_active: false,
            tags,
            active_tags: Vec::new(),
            focused_client: None,
        }
    }

    fn client(id: u32, monitor: &str, tag: u32) -> Client {
        Client {
            id: ClientId::new(id),
            title: None,
            app_id: None,
            monitor: monitor.to_owned(),
            tags: vec![TagId::new(tag)],
            is_urgent: false,
            is_focused: false,
        }
    }

    fn indices(displayed: &[Tag]) -> Vec<u32> {
        displayed.iter().map(|tag| tag.index.get()).collect()
    }

    #[test]
    fn hide_empty_off_shows_every_tag() {
        let displayed = displayed_tags(&monitor("DP-1", nine_tags(1, 1)), false, 0);
        assert_eq!(indices(&displayed), (1..=9).collect::<Vec<_>>());
    }

    #[test]
    fn hide_empty_keeps_only_active_or_occupied() {
        let displayed = displayed_tags(&monitor("DP-1", nine_tags(1, 1)), true, 0);
        assert_eq!(indices(&displayed), vec![1]);
    }

    #[test]
    fn min_tag_count_forces_low_tags_even_when_empty() {
        let displayed = displayed_tags(&monitor("DP-1", nine_tags(1, 1)), true, 5);
        assert_eq!(indices(&displayed), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn clients_scoped_to_their_own_monitor() {
        let clients = vec![
            client(6, "DP-1", 1),
            client(3, "DP-2", 6),
            client(2, "DP-3", 4),
        ];
        let scoped = clients_for_monitor(clients, "DP-1");
        let ids: Vec<u32> = scoped.iter().map(|client| client.id.get()).collect();
        assert_eq!(ids, vec![6]);
    }
}
