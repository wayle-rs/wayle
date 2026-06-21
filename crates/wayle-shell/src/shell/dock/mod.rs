mod adapter;
mod adapter_hyprland;
mod event_watcher;
mod factory;
mod settings;
mod watchers;

use std::collections::HashMap;
use adapter::OpenPopoverTracker;
use tracing::{debug, info};
pub(crate) use adapter::DockAdapterRef;
use adapter::DockAdapter;

use factory::*;
use gtk::prelude::*;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use relm4::{factory::FactoryVecDeque, gtk, gtk::gdk, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::dock::{DockPosition, DockVisibility},
};
use wayle_widgets::watch;

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
    dock_visibility: DockVisibility,
    dock_position: DockPosition,
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
    DockItemsChanged,
    DockItemsChangedWithEvent(DockEvent),
}

#[derive(Debug)]
pub(crate) enum DockInput {
    DockItemAction(String, DockItemInput),
    InitialReady,
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
            add_css_class: "dock",
            set_size_request: (1, 1),

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
        let size = config.dock.size.get();

        let monitor_name = init.monitor.connector().map(|s| s.to_string());

        let open_popover = adapter::create_open_popover_tracker();

        let settings = settings::DockSettings {
            theme_provider: config.styling.theme_provider.clone(),
            icon_position: config.bar.button_icon_position.clone(),
            item_rounding: config.dock.item_rounding.clone(),
            item_padding: config.dock.item_padding.clone(),
            size: ConfigProperty::new(size),
            monitor_name: monitor_name.clone(),
        };

        root.init_layer_shell();
        root.set_layer(gtk4_layer_shell::Layer::Top);
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_monitor(Some(&init.monitor));
        root.set_hexpand(false);
        root.set_vexpand(false);
        Self::apply_dock_anchors(&root, position);
        Self::apply_dock_css_classes(&root, Some(&init.monitor), position);

        let window = root.clone();
        init.monitor.connect_invalidate(move |_| {
            window.destroy();
        });

        let widgets = view_output!();
        let items = FactoryVecDeque::builder()
            .launch(widgets.dock_box.clone())
            .forward(
                sender.input_sender(),
                |output: (String, factory::DockItemInput)| {
                    DockInput::DockItemAction(output.0, output.1)
                },
            );

        let css_provider = gtk::CssProvider::new();

        #[allow(deprecated)]
        root.style_context()
            .add_provider(&css_provider, gtk::STYLE_PROVIDER_PRIORITY_USER);
        css::spawn(&sender, &init.services);
        config::spawn(&sender, &init.services);

        let model = Self {
            settings,
            open_popover: open_popover.clone(),
            services: init.services.clone(),
            css_provider,
            last_css: String::new(),
            items,
            dock_visibility: visibility,
            dock_position: position,
            running_apps: wayle_core::Property::new(Vec::new()),
            adapter: build_adapter(&init.services),
        };

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
            info!(dock = "init", event_watcher = "spawned", "Event watcher started");
            sender.input(DockInput::InitialReady);
        }

        let pinned_stream = config.dock.pinned_apps.watch();
        watch!(sender, [pinned_stream], |out| {
            let _ = out.send(DockCmd::DockItemsChanged);
        });

        let dock_items_widget = model.items.widget();
        dock_items_widget.set_hexpand(false);
        dock_items_widget.set_vexpand(false);
        dock_items_widget.set_halign(gtk::Align::Center);
        widgets.dock_box.set_halign(gtk::Align::Center);
        widgets.dock_box.append(dock_items_widget);
        root.auto_exclusive_zone_enable();

        debug!(
            dock = "init",
            visibility = ?visibility,
            monitor_connector = ?init.monitor.connector(),
            dock_visible = matches!(visibility, DockVisibility::AlwaysVisible),
            "Dock window visibility set"
        );

        root.set_visible(matches!(visibility, DockVisibility::AlwaysVisible));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            DockInput::DockItemAction(app_id, action) => {
                self.handle_dock_item_action(&app_id, action);
            }
            DockInput::InitialReady => {
                debug!(dock = "init", "InitialReady: running apps set, building dock items");
                self.rebuild_all_items();
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
                root.set_visible(matches!(visibility, DockVisibility::AlwaysVisible));
                self.rebuild_all_items();
            }
            DockCmd::PositionChanged => {
                let config = self.services.config.config();
                let new_position = config.dock.position.get();
                if new_position != self.dock_position {
                    Self::apply_dock_anchors(root, new_position);
                    Self::apply_dock_css_classes(root, root.monitor().as_ref(), new_position);
                    self.dock_position = new_position;
                }
            }
            DockCmd::DockItemsChanged => {
                debug!(dock = "cmd", cmd = "DockItemsChanged", "Handling DockItemsChanged command");
                self.rebuild_all_items_from_compositor();
            }
            DockCmd::DockItemsChangedWithEvent(event) => {
                match event {
                    DockEvent::ActiveWindowChanged(focused_id) => {
                        self.incremental_update_focus(focused_id);
                    }
                    DockEvent::WindowOpened | DockEvent::WindowClosed => {
                        self.rebuild_all_items_from_compositor();
                    }
                    DockEvent::WindowsChanged => {
                        self.rebuild_all_items_from_compositor();
                    }
                }
            }
        }
    }
}

fn build_dock_items(
    services: &ShellServices,
    running: &[DockAppData],
) -> Vec<DockItemData> {
    let config = services.config.config();
    let pinned: Vec<String> = config.dock.pinned_apps.get();

    let running_map: HashMap<&str, &DockAppData> =
        running.iter().map(|a| (a.app_id.as_str(), a)).collect();

    let mut items: Vec<DockItemData> = pinned
        .iter()
        .filter_map(|app_id| {
            running_map.get(app_id.as_str()).copied().map(|ra| DockItemData {
                app_id: app_id.clone(),
                is_pinned: true,
                is_running: true,
                is_active: ra.is_active,
                window_count: ra.window_count,
            })
        })
        .collect();

    let running_ids: HashMap<&str, ()> =
        running.iter().map(|a| (a.app_id.as_str(), ())).collect();

    for app_id in &pinned {
        if !running_ids.contains_key(app_id.as_str()) {
            items.push(DockItemData {
                app_id: app_id.clone(),
                is_pinned: true,
                is_running: false,
                is_active: false,
                window_count: 0,
            });
        }
    }

    let pinned_set: HashMap<&str, ()> =
        pinned.iter().map(|s| (s.as_str(), ())).collect();

    for app in running.iter() {
        if !pinned_set.contains_key(app.app_id.as_str()) {
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
    } else if let Some(ref hyprland) = services.hyprland {
        Some(DockAdapterRef::Hyprland(
            crate::shell::dock::adapter_hyprland::HyprlandDockAdapter::new(
                hyprland.clone(),
            ),
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
    ) {
        if let Some(monitor) = monitor {
            if let Some(connector) = monitor.connector() {
                window.add_css_class(&connector.to_string());
                window.set_namespace(Some(&format!("wayle-dock-{}", connector)));
            }
        }

        let class = match position {
            DockPosition::Bottom => "dock-bottom",
            DockPosition::Left => "dock-left",
            DockPosition::Right => "dock-right",
        };
        window.add_css_class(class);
    }

    fn build_css(&self) -> String {
        let config = self.services.config.config();
        let dock = &config.dock;

        let bg = wayle_widgets::styling::resolve_color(
            &dock.bg,
            matches!(
                config.styling.theme_provider.get(),
                wayle_config::schemas::styling::ThemeProvider::Wayle
            ),
        );
        let bg_opacity = dock.background_opacity.get().value() as f64 / 100.0;

        let (r, g, b) = Self::hex_to_rgb(&bg);
        let bg_rgba = format!("rgba({}, {}, {}, {})", r, g, b, bg_opacity);

        format!(
            ".dock {{ \
            background: none; \
            border-radius: 18px; \
            }} \
            .dock .dock-section {{ \
            background-color: {}; \
            border-radius: 18px; \
            padding: 4px 6px; \
            margin: 0; \
            }} \
            .dock .dock-item {{ \
            background-color: transparent; \
            min-width: 24px; \
            min-height: 24px; \
            padding: 0; \
            margin: 4px 2px; \
            border: none; \
            border-radius: 10px; \
            transition: background-color 0.15s ease; \
            }} \
            .dock .dock-item:hover {{ \
            background-color: rgba(255, 255, 255, 0.1); \
            border-radius: 12px; \
            }} \
            .dock .dock-item.active {{ \
            background-color: rgba(255, 255, 255, 0.15); \
            }} \
            .dock .dock-window-popover {{ \
            border-radius: 12px; \
            padding: 6px; \
            }} \
            .dock .dock-popover-content {{ \
            padding: 4px; \
            }} \
            .dock .dock-window-item {{ \
            background-color: transparent; \
            border: none; \
            border-radius: 6px; \
            padding: 4px 10px; \
            margin: 2px 0; \
            font-size: 13px; \
            color: #ffffff; \
            text-align: start; \
            }} \
            .dock .dock-window-item:hover {{ \
            background-color: rgba(255, 255, 255, 0.1); \
            }}",
            bg_rgba
        )
    }

    fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return (0, 0, 0);
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
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

        let mut guard = self.items.guard();
        guard.clear();

        for item in new_items {
            guard.push_back(item);
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
            let mut app_map: std::collections::HashMap<String, DockAppData> =
                new_apps.into_iter().map(|a| (a.app_id.clone(), a)).collect();
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
                debug!(dock = "rebuild", "No change in running apps, skipping rebuild");
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
}
