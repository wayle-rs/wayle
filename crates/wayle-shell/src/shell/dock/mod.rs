mod adapter;
mod adapter_hyprland;
mod event_watcher;
mod factory;
mod icon_resolver;
mod settings;
mod watchers;

use std::collections::HashMap;
use std::rc::Rc;

pub(crate) use adapter::DockAdapterRef;
use adapter::{DockAdapter, OpenPopoverTracker};
use factory::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk, prelude::*};
use tracing::{debug, info};
use wayle_config::{
    ConfigProperty,
    schemas::dock::{DockPosition, DockVisibility},
};
use self::watchers::{config, css};
use crate::shell::services::ShellServices;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DockAppData {
    pub app_id: String,
    pub is_active: bool,
    pub window_count: u32,
}

/// Running application information for dock items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DockItemData {
    pub app_id: String,
    pub is_pinned: bool,
    pub is_running: bool,
    pub is_active: bool,
    pub window_count: u32,
}

pub(crate) struct Dock {
    settings: settings::DockSettings,
    open_popover: OpenPopoverTracker,
    services: ShellServices,
    css_provider: gtk::CssProvider,
    last_css: String,
    items: FactoryVecDeque<DockItem>,
    dock_box: gtk::Box,
    dock_visibility: DockVisibility,
    dock_position: DockPosition,
    hover_active: bool,
    running_apps: wayle_core::Property<Vec<DockAppData>>,
    adapter: Option<DockAdapterRef>,
}

pub(crate) struct DockInit {
    pub(crate) monitor: gdk::Monitor,
    pub(crate) services: ShellServices,
}

/// Dock-relevant compositor event type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DockEvent {
    /// Window focus changed — dock should update active state.
    ActiveWindowChanged(Option<String>),
    /// A new window appeared.
    WindowOpened,
    /// A window was closed.
    WindowClosed,
    /// Generic change that requires full rebuild (resize, move, layout, etc).
    WindowsChanged,
}

#[derive(Debug)]
pub(crate) enum DockCmd {
    StyleChanged,
    PositionChanged,
    VisibilityChanged,
    DockItemsChanged,
    DockItemsChangedWithEvent(DockEvent),
}

#[derive(Debug)]
pub(crate) enum DockInput {
    DockItemAction(String, DockItemInput),
    InitialReady,
    HoverEnter,
    HoverLeave,
}

#[relm4::component(pub(crate))]
impl Component for Dock {
    type Init = DockInit;
    type Input = DockInput;
    type Output = (String, DockItemInput);
    type CommandOutput = DockCmd;

    view! {
        #[root]
        gtk::Window {
            set_decorated: false,
            set_size_request: (1, 1),
            add_css_class: "dock",

            #[name = "dock_box"]
            gtk::Box {
                add_css_class: "dock-section",
                set_size_request: (1, 1),
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.services.config.config();
        let position = config.dock.position.get();
        let visibility = config.dock.visibility.get();

        // Phase 1: Config reads
        let open_popover = adapter::create_open_popover_tracker();
        let settings = settings::DockSettings {
            item_rounding: config.dock.item_rounding.clone(),
            item_padding: config.dock.item_padding.clone(),
            size: ConfigProperty::new(config.dock.size.get()),
            dock_position: position,
            active_border_width: config.dock.active_border_width.clone(),
            active_border_color: config.dock.active_border_color.clone(),
        };
        let adapter = build_adapter(&init.services);
        let css_provider = gtk::CssProvider::new();

        // Phase 2: UI setup
        Self::setup_dock_window(&root, &init.monitor, position, visibility);

        #[allow(deprecated)]
        root.style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);

        let initial_css = Self::build_initial_css(&settings, &init.services);
        css_provider.load_from_string(&initial_css);

        let widgets = view_output!();
        let items = FactoryVecDeque::builder()
            .launch(widgets.dock_box.clone())
            .forward(
                sender.input_sender(),
                |output: (String, factory::DockItemInput)| {
                    DockInput::DockItemAction(output.0, output.1)
                },
            );

        let hover_state = Rc::new(std::cell::Cell::new(false));

        // Phase 3: Model + watchers
        let model = Self {
            settings,
            open_popover: open_popover.clone(),
            services: init.services.clone(),
            css_provider,
            last_css: initial_css,
            items,
            dock_box: widgets.dock_box.clone(),
            dock_visibility: visibility,
            dock_position: position,
            hover_active: false,
            running_apps: wayle_core::Property::new(Vec::new()),
            adapter,
        };

        let dock_items_widget = model.items.widget();
        dock_items_widget.set_hexpand(false);
        dock_items_widget.set_vexpand(false);
        widgets.dock_box.set_halign(gtk::Align::Fill);
        widgets.dock_box.set_valign(gtk::Align::Fill);
        Self::apply_dock_box_alignment(&widgets.dock_box, position);
        Self::apply_dock_box_orientation(&widgets.dock_box, position);

   

      // Add hover controller for autohide
        let hover_state = hover_state.clone();
        let hover_sender = sender.clone();
        {
            let motion_controller = gtk::EventControllerMotion::new();
            let hover_state_enter = hover_state.clone();
            let hover_sender_enter = hover_sender.clone();
            motion_controller.connect_enter(move |_controller, _x, _y| {
                if hover_state_enter.get() {
                    return;
                }
                hover_state_enter.set(true);
                hover_sender_enter.input(DockInput::HoverEnter);
            });
            let hover_state_leave = hover_state.clone();
            let hover_sender_leave = hover_sender.clone();
            motion_controller.connect_leave(move |_controller| {
                if !hover_state_leave.get() {
                    return;
                }
                hover_state_leave.set(false);
                hover_sender_leave.input(DockInput::HoverLeave);
            });
            root.add_controller(motion_controller);
        }

        if let Some(ref adapter) = model.adapter {
            let initial_apps = adapter.compute_running_apps();
            debug!(
                dock = "init",
                initial_app_count = initial_apps.len(),
                app_ids = ?initial_apps.iter().map(|a| &a.app_id).collect::<Vec<_>>(),
                "Initial running apps computed"
            );
            model.running_apps.set(initial_apps.clone());
            event_watcher::spawn(&sender, &init.services, adapter.clone());
            info!(
                dock = "init",
                event_watcher = "spawned",
                "Event watcher started"
            );
        }

        css::spawn(&sender, &init.services);
        config::spawn(&sender, &init.services);

        if model.adapter.is_some() {
            sender.input(DockInput::InitialReady);
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            DockInput::DockItemAction(app_id, action) => {
                self.handle_dock_item_action(&app_id, action);
            }
            DockInput::InitialReady => {
                debug!(
                    dock = "init",
                    "InitialReady: running apps set, building dock items"
                );
                self.rebuild_all_items();
            }
            DockInput::HoverEnter => {
                self.enter_hover(root);
            }
            DockInput::HoverLeave => {
                self.leave_hover(root);
            }
        }
    }

    fn update_cmd(&mut self, msg: DockCmd, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            DockCmd::StyleChanged => {
                let config = self.services.config.config();
                let visibility = config.dock.visibility.get();
                self.dock_visibility = visibility;
                let new_css = self.build_css();
                if new_css != self.last_css {
                    self.css_provider.load_from_string(&new_css);
                    self.last_css = new_css;
                }
                self.rebuild_all_items();
                root.set_visible(true);
            }
            DockCmd::PositionChanged => {
                let config = self.services.config.config();
                let new_position = config.dock.position.get();
                if new_position != self.dock_position {
                    Self::apply_dock_anchors(root, new_position);
                    Self::apply_dock_css_classes(root, root.monitor().as_ref(), new_position, self.dock_visibility);
                    Self::apply_dock_box_alignment(&self.dock_box, new_position);
                    Self::apply_dock_box_orientation(&self.dock_box, new_position);
                    self.settings.dock_position = new_position;

                    // Re-apply autohide sliver if in autohide mode
                    if self.dock_visibility == DockVisibility::Autohide {
                        Self::apply_autohide_sliver(root, &root.monitor().expect("monitor"), new_position);
                    }

                    self.rebuild_all_items();
                    self.dock_position = new_position;
                }
            }
            DockCmd::VisibilityChanged => {
                let config = self.services.config.config();
                let visibility = config.dock.visibility.get();
                let was_autohide = self.dock_visibility == DockVisibility::Autohide;
                self.dock_visibility = visibility;

                match visibility {
                    DockVisibility::AlwaysVisible => {
                        root.auto_exclusive_zone_enable();
                        if was_autohide {
                            let dock_size = config.dock.size.get();
                            if let Some(monitor) = root.monitor() {
                                Self::restore_full_size(root, &monitor, self.dock_position, dock_size);
                            }
                        }
                    }
                    DockVisibility::Autohide => {
                        root.set_exclusive_zone(0);
                        if let Some(monitor) = root.monitor() {
                            Self::apply_autohide_sliver(root, &monitor, self.dock_position);
                        }
                    }
                }

                Self::apply_dock_css_classes(root, root.monitor().as_ref(), self.dock_position, visibility);

                let new_css = self.build_css();
                if new_css != self.last_css {
                    self.css_provider.load_from_string(&new_css);
                    self.last_css = new_css;
                }

                root.set_visible(true);
            }
            DockCmd::DockItemsChanged => {
                debug!(
                    dock = "cmd",
                    cmd = "DockItemsChanged",
                    "Handling DockItemsChanged command"
                );
                self.rebuild_all_items_from_compositor();
            }
            DockCmd::DockItemsChangedWithEvent(event) => match event {
                DockEvent::ActiveWindowChanged(focused_id) => {
                    self.incremental_update_focus(focused_id);
                }
                DockEvent::WindowOpened | DockEvent::WindowClosed => {
                    self.rebuild_all_items_from_compositor();
                }
                DockEvent::WindowsChanged => {
                    self.rebuild_all_items_from_compositor();
                }
            },
        }
    }
}

fn build_dock_items(services: &ShellServices, running: &[DockAppData]) -> Vec<DockItemData> {
    let config = services.config.config();
    let pinned: Vec<String> = config.dock.pinned_apps.get();

    let running_map: HashMap<&str, &DockAppData> =
        running.iter().map(|a| (a.app_id.as_str(), a)).collect();
    let pinned_set: std::collections::HashSet<&str> = pinned.iter().map(|s| s.as_str()).collect();

    let mut items: Vec<DockItemData> = pinned
        .iter()
        .filter_map(|app_id| {
            running_map
                .get(app_id.as_str())
                .copied()
                .map(|ra| DockItemData {
                    app_id: app_id.clone(),
                    is_pinned: true,
                    is_running: true,
                    is_active: ra.is_active,
                    window_count: ra.window_count,
                })
                .or_else(|| Some(DockItemData {
                    app_id: app_id.clone(),
                    is_pinned: true,
                    is_running: false,
                    is_active: false,
                    window_count: 0,
                }))
        })
        .collect();

    for app in running.iter() {
        if !pinned_set.contains(app.app_id.as_str()) {
            items.push(DockItemData {
                app_id: app.app_id.clone(),
                is_pinned: false,
                is_running: true,
                is_active: app.is_active,
                window_count: app.window_count,
            });
        }
    }

    items
}

fn build_adapter(services: &ShellServices) -> Option<DockAdapterRef> {
    if let Some(ref niri) = services.niri {
        Some(DockAdapterRef::Niri(
            crate::shell::dock::adapter::NiriDockAdapter::new(niri.clone()),
        ))
    } else {
        None
    }
}

impl Dock {
    fn handle_dock_item_action(&self, app_id: &str, _: DockItemInput) {
        if let Some(ref adapter) = self.adapter {
            adapter.focus_app(app_id);
        }
    }

    fn apply_dock_anchors(window: &gtk::Window, position: DockPosition) {
        window.set_anchor(gtk4_layer_shell::Edge::Top, false);
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
        window.set_anchor(gtk4_layer_shell::Edge::Left, false);
        window.set_anchor(gtk4_layer_shell::Edge::Right, false);

        window.set_margin(gtk4_layer_shell::Edge::Top, 0);
        window.set_margin(gtk4_layer_shell::Edge::Bottom, 0);
        window.set_margin(gtk4_layer_shell::Edge::Left, 0);
        window.set_margin(gtk4_layer_shell::Edge::Right, 0);

        let dock_margin = 6;

        match position {
            DockPosition::Bottom => {
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_margin(gtk4_layer_shell::Edge::Bottom, dock_margin);
            }
            DockPosition::Left => {
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            }
            DockPosition::Right => {
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
            }
        }
    }

    fn apply_dock_css_classes(
        window: &gtk::Window,
        monitor: Option<&gdk::Monitor>,
        position: DockPosition,
        visibility: DockVisibility,
    ) {
        if let Some(monitor) = monitor
            && let Some(connector) = monitor.connector()
        {
            window.add_css_class(&connector);
            window.set_namespace(Some(&format!("wayle-dock-{}", connector)));
        }

        let class = match position {
            DockPosition::Bottom => "horizontal",
            DockPosition::Left => "vertical",
            DockPosition::Right => "vertical",
        };
        window.add_css_class(class);

        if visibility == DockVisibility::Autohide {
            window.add_css_class("autohide");
        } else {
            window.remove_css_class("autohide");
        }
    }

    fn setup_dock_window(
        window: &gtk::Window,
        monitor: &gdk::Monitor,
        position: DockPosition,
        visibility: DockVisibility,
    ) {
        window.init_layer_shell();
        window.set_layer(gtk4_layer_shell::Layer::Top);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_monitor(Some(monitor));
        window.set_hexpand(false);
        window.set_vexpand(false);
        Self::apply_dock_anchors(window, position);
        Self::apply_dock_css_classes(window, Some(monitor), position, visibility);

        match visibility {
            DockVisibility::AlwaysVisible => {
                window.auto_exclusive_zone_enable();
            }
            DockVisibility::Autohide => {
                window.set_exclusive_zone(0);
                Self::apply_autohide_sliver(window, monitor, position);
            }
        }

        window.set_visible(true);

        let win = window.clone();
        monitor.connect_invalidate(move |_| {
            win.destroy();
        });
    }

    fn build_css(&self) -> String {
        Self::build_css_from_settings(&self.settings, &self.services)
    }

    fn build_initial_css(settings: &settings::DockSettings, services: &ShellServices) -> String {
        Self::build_css_from_settings(settings, services)
    }

    fn build_css_from_settings(
        settings: &settings::DockSettings,
        services: &ShellServices,
    ) -> String {
        let config = services.config.config();
        let dock = &config.dock;
        let is_wayle = matches!(
            config.styling.theme_provider.get(),
            wayle_config::schemas::styling::ThemeProvider::Wayle
        );

        let bg = wayle_widgets::styling::resolve_color(&dock.bg, is_wayle);
        let bg_opacity = dock.background_opacity.get().value();
        let size = settings.size.get();
        let padding = settings.item_padding.get();
        let rounding = settings.item_rounding.get();
        let active_border_width = settings.active_border_width.get();
        let active_border_color = wayle_widgets::styling::resolve_color(&dock.active_border_color, is_wayle);

        let border_radius = match rounding {
            wayle_config::schemas::styling::RoundingLevel::None => 0,
            wayle_config::schemas::styling::RoundingLevel::Sm => 6,
            wayle_config::schemas::styling::RoundingLevel::Md => 10,
            wayle_config::schemas::styling::RoundingLevel::Lg => 14,
            wayle_config::schemas::styling::RoundingLevel::Full => 18,
        };

        let section_padding = (padding.value() * 8.0).round() as i32;
        let section_gap = ((padding.value() * 8.0 + 4.0) * 2.0).round() as i32;
        let popover_item_padding = ((padding.value() * 8.0 + 4.0) * 2.0).round() as i32;

        format!(
            ".dock {{ \
            --dock-bg: {bg}; \
            --dock-opacity: {bg_opacity}%; \
            --dock-section-padding-px: {section_padding}; \
            --dock-section-gap-px: {section_gap}; \
            --dock-item-size-px: {size}; \
            --dock-item-border-radius: {border_radius}px; \
            --dock-border-radius: {border_radius}px; \
            --dock-popover-item-padding-px: {popover_item_padding}; \
            --dock-item-unpinned-opacity: 0.7; \
            --dock-active-border-width: {active_border_width}px; \
            --dock-active-border-color: {active_border_color}; \
            --dock-item-hover-bg: color-mix(in srgb, var(--bg-overlay) 50%, transparent); \
            --dock-item-hover-border: var(--border-subtle); \
            --dock-popover-bg: var(--bg-elevated); \
            }}"
        )
    }

    fn build_dock_items(&self) -> Vec<DockItemData> {
        let running = self.running_apps.get();
        build_dock_items(&self.services, &running)
    }

    fn rebuild_all_items(&mut self) {
        let settings = self.settings.clone();
        let config = self.services.config.config();
        let show_running = config.dock.show_running.get();

        let dock_items = self.build_dock_items();

        let new_items: Vec<factory::DockItemInit> = dock_items
            .into_iter()
            .filter(|item| show_running || item.is_pinned)
            .map(|item| factory::DockItemInit {
                app_id: item.app_id,
                is_pinned: item.is_pinned,
                is_running: item.is_running,
                is_active: item.is_active,
                settings: settings.clone(),
                adapter: self.adapter.clone(),
                open_popover: self.open_popover.clone(),
            })
            .collect();

        let old_count = self.items.len();
        debug!(old_count, "rebuild_all_items START");
        *self.open_popover.borrow_mut() = None;
        debug!("rebuild_all_items tracker cleared");

        {
            let mut guard = self.items.guard();
            debug!("rebuild_all_items clearing items");
            guard.clear();
            for item in new_items {
                guard.push_back(item);
            }
        }
    }

    fn rebuild_all_items_from_compositor(&mut self) {
        if let Some(ref adapter) = self.adapter {
            let new_apps: Vec<DockAppData> = adapter.compute_running_apps();
            debug!(
                dock = "rebuild",
                new_app_count = new_apps.len(),
                app_ids = ?new_apps.iter().map(|a| &a.app_id).collect::<Vec<_>>(),
                "Rebuilding dock items from compositor state"
            );
            let mut app_map: std::collections::HashMap<String, DockAppData> = new_apps
                .into_iter()
                .map(|a| (a.app_id.clone(), a))
                .collect();
            let old_apps = self.running_apps.get();
            let mut ordered: Vec<DockAppData> = Vec::new();
            let mut changed = false;
            for app in old_apps.iter() {
                if let Some(updated) = app_map.remove(&app.app_id) {
                    changed = changed || updated != *app;
                    ordered.push(updated);
                } else {
                    changed = true;
                }
            }
            if !app_map.is_empty() {
                changed = true;
            }
            ordered.extend(app_map.into_values());
            if changed {
                debug!(
                    dock = "rebuild",
                    old_app_count = old_apps.len(),
                    new_app_count = ordered.len(),
                    "Running apps changed, rebuilding dock items"
                );
                self.running_apps.set(ordered);
                self.rebuild_all_items();
            } else {
                debug!(
                    dock = "rebuild",
                    "No change in running apps, skipping rebuild"
                );
            }
        }
    }

    fn incremental_update_focus(&mut self, focused_id: Option<String>) {
        debug!(
            dock = "focus",
            focused_app_id = ?focused_id,
            app_count = self.running_apps.get().len(),
            "Incremental focus update"
        );
        let mut old_apps = self.running_apps.get();
        let mut changed = false;
        if let Some(ref focused_id) = focused_id {
            for app in old_apps.iter_mut() {
                if app.app_id == *focused_id {
                    if !app.is_active {
                        app.is_active = true;
                        changed = true;
                    }
                } else if app.is_active {
                    app.is_active = false;
                    changed = true;
                }
            }
        }
        if changed {
            self.running_apps.set(old_apps);
            self.rebuild_all_items();
        }
    }

    fn apply_dock_box_alignment(dock_box: &gtk::Box, position: DockPosition) {
        match position {
            DockPosition::Bottom => {
                dock_box.set_halign(gtk::Align::Center);
                dock_box.set_valign(gtk::Align::Fill);
            }
            DockPosition::Left | DockPosition::Right => {
                dock_box.set_halign(gtk::Align::Fill);
                dock_box.set_valign(gtk::Align::Center);
            }
        }
    }

    fn apply_dock_box_orientation(dock_box: &gtk::Box, position: DockPosition) {
        let orientation = match position {
            DockPosition::Bottom => gtk::Orientation::Horizontal,
            DockPosition::Left | DockPosition::Right => gtk::Orientation::Vertical,
        };
        dock_box.set_orientation(orientation);
    }

    fn apply_autohide_sliver(window: &gtk::Window, monitor: &gdk::Monitor, position: DockPosition) {
        let sliver_size = 3;
        let geom = monitor.geometry();
        match position {
            DockPosition::Bottom => {
                window.set_size_request(geom.width(), sliver_size);
            }
            DockPosition::Left | DockPosition::Right => {
                window.set_size_request(sliver_size, geom.height());
            }
        }
    }

    fn restore_full_size(window: &gtk::Window, monitor: &gdk::Monitor, position: DockPosition, size_px: u32) {
        let margin = 6;
        let geom = monitor.geometry();
        match position {
            DockPosition::Bottom => {
                window.set_size_request(geom.width(), size_px as i32 + margin * 2);
            }
            DockPosition::Left | DockPosition::Right => {
                window.set_size_request(size_px as i32 + margin * 2, geom.height());
            }
        }
    }

   fn enter_hover(&mut self, root: &gtk::Window) {
        if self.dock_visibility != DockVisibility::Autohide {
            return;
        }
        if self.hover_active {
            return;
        }
        self.hover_active = true;

        let config = self.services.config.config();
        let dock_size = config.dock.size.get();
        if let Some(monitor) = root.monitor() {
            Self::restore_full_size(root, &monitor, self.dock_position, dock_size);
            root.remove_css_class("autohide");
        }
    }

    fn leave_hover(&mut self, root: &gtk::Window) {
        if self.dock_visibility != DockVisibility::Autohide {
            return;
        }
        if !self.hover_active {
            return;
        }
        self.hover_active = false;

        if let Some(monitor) = root.monitor() {
            let position = self.dock_position;
            Self::apply_autohide_sliver(root, &monitor, position);
            root.add_css_class("autohide");
        }
    }
}
