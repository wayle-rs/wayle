//! App data model and factory builders for the app picker.

mod row;

pub(crate) use row::app_picker;

use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;

use relm4::gtk;
use relm4::gtk::pango;
use relm4::gtk::prelude::*;
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::gtk::prelude::WidgetExt;

use gio_unix::DesktopAppInfo;
use wayle_config::ConfigProperty;

use super::{WatcherHandle, spawn_property_watcher};

/// A single application discovered from the desktop database.
pub(super) struct AppInfo {
    /// Display name (e.g. "Firefox", "GNOME Console") — what the user sees and searches.
    pub(super) display_name: String,
    /// App ID without .desktop suffix (e.g. "firefox", "org.gnome.Console") — stored in config.
    pub(super) app_id: String,
    /// Icon name from .desktop file. Falls back to app_id if no icon available.
    pub(super) icon: String,
}

/// Lookup table for app icons keyed by display name.
#[derive(Clone)]
pub(super) struct AppLookup {
    /// display_name → icon_name mapping for icon resolution.
    pub(super) icons: HashMap<String, String>,
    /// display_name → app_id mapping for activation handling.
    pub(super) app_ids: HashMap<String, String>,
}

/// Reverse lookup: app_id → display_name for chip building.
#[derive(Clone)]
pub(super) struct AppIdMap {
    /// app_id → display_name mapping.
    pub(super) name: HashMap<String, String>,
}

/// Enumerate all desktop applications and return them as `AppInfo` structs.
pub(super) fn enumerate_desktop_apps() -> Vec<AppInfo> {
    gtk::gio::AppInfo::all()
        .into_iter()
        .filter_map(|app_info| {
            let desktop_app = app_info.downcast_ref::<DesktopAppInfo>()?;

            let display_name = desktop_app.name().to_string();
            if display_name.is_empty() {
                return None;
            }

            let raw_id = desktop_app.id();
            let app_id = raw_id
                .map(|id| id.strip_suffix(".desktop").unwrap_or(&id).to_string())
                .unwrap_or_default();

            let icon = desktop_app.icon()
                .and_then(|icon| icon.to_string())
                .unwrap_or_else(|| app_id.clone().into());

            Some(AppInfo {
                display_name,
                app_id,
                icon: icon.to_string(),
            })
        })
        .collect()
}

/// Build a `PropertyExpression` for the app name (display name) in a `StringObject`.
pub(super) fn build_name_expression() -> gtk::PropertyExpression {
    gtk::PropertyExpression::new(
        gtk::StringObject::static_type(),
        gtk::Expression::NONE,
        "string",
    )
}

/// Build a `StringFilter` for substring-based, case-insensitive app name matching.
pub(super) fn build_filter(expression: &gtk::PropertyExpression) -> gtk::StringFilter {
    gtk::StringFilter::builder()
        .expression(expression.clone())
        .match_mode(gtk::StringFilterMatchMode::Substring)
        .ignore_case(true)
        .build()
}

/// Build a `StringList`, `AppLookup`, and `AppIdMap` from the given list of `AppInfo`.
pub(super) fn build_app_list(apps: &[AppInfo]) -> (gtk::StringList, AppLookup, AppIdMap) {
    let names: Vec<&str> = apps.iter().map(|a| a.display_name.as_str()).collect();
    let string_list = gtk::StringList::new(&names);

    let icons = apps.iter()
        .map(|a| (a.display_name.clone(), a.icon.clone()))
        .collect();

    let app_ids = apps.iter()
        .map(|a| (a.display_name.clone(), a.app_id.clone()))
        .collect();

    let name_map = apps.iter()
        .map(|a| (a.app_id.clone(), a.display_name.clone()))
        .collect();

    (string_list, AppLookup { icons, app_ids }, AppIdMap { name: name_map })
}

/// Build a list-item factory that renders each app row with icon, name, and pinned indicator.
pub(super) fn build_factory(
    lookup: AppLookup,
    pinned_apps: Rc<RefCell<HashSet<String>>>,
) -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let box_ = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .valign(gtk::Align::Center)
            .build();

        let icon = gtk::Image::builder()
            .icon_name("application-x-executable")
            .build();
        icon.add_css_class("app-picker-icon");
        box_.append(&icon);

        let name_label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .hexpand(true)
            .ellipsize(pango::EllipsizeMode::End)
            .xalign(0.0)
            .build();
        name_label.add_css_class("app-picker-name");
        box_.append(&name_label);

        let pinned_badge = gtk::Label::builder()
            .label("✓")
            .visible(false)
            .halign(gtk::Align::End)
            .build();
        pinned_badge.add_css_class("app-picker-pinned");
        box_.append(&pinned_badge);

        list_item.set_child(Some(&box_));
    });

      let pinned_apps = pinned_apps.clone();
    let lookup = Rc::new(lookup);

    factory.connect_bind(move |_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let Some(string_object) = list_item.item().and_downcast::<gtk::StringObject>() else {
            return;
        };

        let display_name = string_object.string().to_string();

        let Some(child) = list_item.child() else {
            return;
        };

        let Some(box_) = child.downcast_ref::<gtk::Box>() else {
            return;
        };

        let children: Vec<gtk::Widget> = {
            let mut list = Vec::new();
            let mut child = box_.first_child();
            while let Some(c) = child {
                list.push(c.clone());
                child = c.next_sibling();
            }
            list
        };

        let mut idx = 0;
        let icon = children.get(idx).and_then(|c| c.downcast_ref::<gtk::Image>());
        idx += 1;
        if let Some(icon) = icon {
            let icon_name = lookup.icons.get(&display_name).cloned()
                .unwrap_or_else(|| "application-x-executable".to_string());
            icon.set_icon_name(Some(&icon_name));
        }

        let name_label = children.get(idx).and_then(|c| c.downcast_ref::<gtk::Label>());
        idx += 1;
        if let Some(name_label) = name_label {
            name_label.set_label(&display_name);
        }

        let pinned_badge = children.get(idx).and_then(|c| c.downcast_ref::<gtk::Label>());
        if let Some(pinned_badge) = pinned_badge {
            let is_pinned = lookup.app_ids.get(&display_name)
                .and_then(|aid| {
                    pinned_apps.try_borrow().ok().map(|p| p.contains(aid))
                })
                .unwrap_or(false);
            pinned_badge.set_visible(is_pinned);
        }
    });

    factory
}

/// Relm4 component for the app picker.
pub(crate) struct AppPickerControl {
    /// The config property being edited.
    property: ConfigProperty<Vec<String>>,
    /// Underlying string list of all app display names.
    string_list: gtk::StringList,
    /// Filter for search queries.
    filter: gtk::StringFilter,
    /// Filtered model backing the ListView.
    filter_model: gtk::FilterListModel,
    /// Search entry widget.
    search_entry: gtk::SearchEntry,
    /// List view widget.
    list_view: gtk::ListView,
    /// Currently pinned app_ids for factory + activation.
    pinned_apps: Rc<RefCell<HashSet<String>>>,
    /// Container holding pinned app chips.
    pinned_container: gtk::Box,
    /// Icon and app_id lookup by display name.
    lookup: AppLookup,
    /// Reverse lookup: app_id → display_name for chips.
    app_id_map: AppIdMap,
    /// Handles the property change watcher.
    _watcher: WatcherHandle,
}

#[derive(Debug)]
pub(crate) enum AppPickerMsg {
    /// User typed in search box.
    SearchChanged(String),
    /// User clicked a list row (position from connect_activate).
    Activated(u32),
    /// External property change — sync UI.
    Refresh,
}

/// Initialization data for [`AppPickerControl`].
pub(crate) struct AppPickerInit {
    /// The config property being edited.
    pub(super) property: ConfigProperty<Vec<String>>,
    /// Dirty badge label from the parent row.
    pub(super) pinned_badge: gtk::Label,
}

impl SimpleComponent for AppPickerControl {
    type Init = AppPickerInit;
    type Input = AppPickerMsg;
    type Output = ();
    type Root = gtk::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk::Box::builder()
            .hexpand(false)
            .valign(gtk::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let apps = enumerate_desktop_apps();

        let (string_list, lookup, app_id_map) = build_app_list(&apps);

        let expression = build_name_expression();
        let filter = build_filter(&expression);

        let filter_model = gtk::FilterListModel::new(
            Some(string_list.clone()),
            Some(filter.clone()),
        );
        let selection = gtk::NoSelection::new(Some(filter_model.clone()));

        let pinned_apps: Rc<RefCell<HashSet<String>>> = Rc::new(RefCell::new(
            init.property.get().into_iter().collect(),
        ));

        let factory = build_factory(lookup.clone(), Rc::clone(&pinned_apps));

        let search_entry = gtk::SearchEntry::builder()
            .placeholder_text("Search applications...")
            .valign(gtk::Align::Center)
            .build();
        if let Some(icon) = search_entry.first_child() {
            icon.set_halign(gtk::Align::Center);
            icon.set_valign(gtk::Align::Center);
            icon.add_css_class("app-picker-search-icon");
        }

        let list_view = gtk::ListView::builder()
            .model(&selection)
            .factory(&factory)
            .single_click_activate(true)
            .show_separators(false)
            .build();

        let scrolled = gtk::ScrolledWindow::builder()
            .child(&list_view)
            .vexpand(true)
            .height_request(150)
            .build();
        scrolled.add_css_class("app-picker-scroll");

        let pinned_container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();
        pinned_container.add_css_class("app-picker-pinned-container");

        root.set_orientation(gtk::Orientation::Vertical);
        root.set_spacing(6);
        root.append(&search_entry);
        root.append(&scrolled);
        root.append(&pinned_container);

        let sender_search = sender.clone();
        search_entry.connect_search_changed(move |entry| {
            sender_search.input(AppPickerMsg::SearchChanged(entry.text().into()));
        });

        let pinned_for_activate = Rc::clone(&pinned_apps);
        let prop_for_activate = init.property.clone();
        let lookup_for_activate = lookup.clone();
        let filter_model_for_activate = filter_model.clone();
        let _activate_handler = list_view.connect_activate(move |_list_view, position| {
            let Some(item) = filter_model_for_activate.item(position).and_downcast::<gtk::StringObject>()
            else {
                return;
            };
            let display_name = item.string().to_string();

            let Some(app_id) = lookup_for_activate.app_ids.get(&display_name) else {
                return;
            };

            {
                let mut pinned = pinned_for_activate.borrow_mut();
                if pinned.contains(app_id) {
                    pinned.remove(app_id);
                } else {
                    pinned.insert(app_id.clone());
                }
                prop_for_activate.set(pinned.iter().cloned().collect::<Vec<_>>());
            }
        });

        let input_sender = sender.input_sender().clone();
        let watcher = spawn_property_watcher(&init.property, move || {
            input_sender.send(AppPickerMsg::Refresh).is_ok()
        });

        let model = Self {
            property: init.property,
            string_list,
            filter,
            filter_model,
            search_entry,
            list_view,
            pinned_apps,
            pinned_container,
            lookup,
            app_id_map,
            _watcher: watcher,
        };
        model.refresh_pinned_chips();

        let widgets = ();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: AppPickerMsg, _sender: ComponentSender<Self>) {
        match msg {
            AppPickerMsg::SearchChanged(query) => {
                self.filter.set_search(Some(&query));
            }

            AppPickerMsg::Activated(_position) => {
                // Activation is handled by `connect_activate` callback above.
            }

            AppPickerMsg::Refresh => {
                let pinned: Vec<String> = self.property.get();
                *self.pinned_apps.borrow_mut() = pinned.into_iter().collect();
                self.refresh_pinned_chips();
            }
        }
    }
}

impl AppPickerControl {
    fn refresh_pinned_chips(&self) {
        while let Some(child) = self.pinned_container.first_child() {
            self.pinned_container.remove(&child);
        }

        let pinned = self.pinned_apps.borrow();
        for app_id in pinned.iter() {
            let chip = self.build_chip(app_id);
            self.pinned_container.append(&chip);
        }
    }

    fn build_chip(&self, app_id: &str) -> gtk::Box {
        let display_name = self.app_id_map.name.get(app_id)
            .cloned()
            .unwrap_or_else(|| app_id.to_string());

        let icon_name = self.lookup.icons.get(display_name.as_str())
            .cloned()
            .unwrap_or_else(|| "application-x-executable".to_string());

        let chip = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .valign(gtk::Align::Center)
            .build();
        chip.add_css_class("app-picker-chip");

        let icon = gtk::Image::builder()
            .icon_name(&icon_name)
            .build();
        chip.append(&icon);

        let label = gtk::Label::builder()
            .label(&display_name)
            .valign(gtk::Align::Center)
            .build();
        label.add_css_class("app-picker-chip-label");
        chip.append(&label);

        let remove_btn = gtk::Button::builder()
            .label("×")
            .valign(gtk::Align::Center)
            .build();
        remove_btn.add_css_class("app-picker-chip-remove");
        let app_id_clone = app_id.to_string();
        let pinned_apps = Rc::clone(&self.pinned_apps);
        let prop = self.property.clone();
        remove_btn.connect_clicked(move |_btn| {
            {
                let mut pinned = pinned_apps.borrow_mut();
                pinned.remove(&app_id_clone);
                prop.set(pinned.iter().cloned().collect::<Vec<_>>());
            }
        });
        chip.append(&remove_btn);

        chip
    }
}
