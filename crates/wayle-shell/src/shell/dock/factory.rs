use std::rc::Rc;

use relm4::{gtk, gtk::prelude::*, prelude::*};
use tracing::debug;

use wayle_config::schemas::dock::DockPosition;

use super::{
    adapter::{self, DockAdapter, DockAdapterRef},
    settings::DockSettings,
};
use super::icon_resolver::resolve_app_icon;

pub(crate) struct DockItem {
    app_id: String,
    is_pinned: bool,
    is_running: bool,
    is_active: bool,
    settings: DockSettings,
    adapter: Option<DockAdapterRef>,
    open_popover: super::OpenPopoverTracker,
    content: Rc<gtk::Box>,
    popover: gtk::Popover,
    _root: gtk::Button,
}

#[derive(Clone)]
pub(crate) struct DockItemInit {
    pub(crate) app_id: String,
    pub(crate) is_pinned: bool,
    pub(crate) is_running: bool,
    pub(crate) is_active: bool,
    pub(crate) settings: DockSettings,
    pub(crate) adapter: Option<DockAdapterRef>,
    pub(crate) open_popover: super::OpenPopoverTracker,
}

#[derive(Debug)]
pub(crate) enum DockItemInput {
    Click,
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for DockItem {
    type Init = DockItemInit;
    type Input = DockItemInput;
    type Output = (String, DockItemInput);
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            add_css_class: "dock-item",
            set_focusable: false,

            #[name = "icon_box"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,

                #[name = "icon_image"]
                gtk::Image {
                    set_pixel_size: 24,
                    set_halign: gtk::Align::Center,
                },

                #[name = "indicator_revealer"]
                gtk::Revealer {
                    set_reveal_child: false,
                    set_transition_type: gtk::RevealerTransitionType::Crossfade,

                    #[name = "indicator_dot"]
                    gtk::Box {
                        add_css_class: "dock-indicator",
                        set_hexpand: true,
                        set_vexpand: false,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::End,
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let content = Rc::new(gtk::Box::new(gtk::Orientation::Vertical, 0));
        let popover = gtk::Popover::new();

        Self {
            app_id: init.app_id,
            is_pinned: init.is_pinned,
            is_running: init.is_running,
            is_active: init.is_active,
            settings: init.settings,
            adapter: init.adapter,
            open_popover: init.open_popover,
            content,
            popover,
            _root: gtk::Button::new(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        self._root = root.clone();
        self.update_model(&widgets);

        self.popover.set_has_arrow(false);

        // Content box as popover child - populated on hover
        let content = self.content.clone();
        content.add_css_class("dock-popover-content");
        content.set_size_request(180, -1);
        self.popover.add_css_class("dock-window-popover");
        self.popover.set_child(Some(&*content));

        // Left-click - close popover first (if open), then focus app
        let sender_clone = sender.clone();
        let tracker = self.open_popover.clone();
        root.connect_clicked(move |_| {
            debug!("CLICK clearing tracker");
            if let Some(ref old) = *tracker.borrow() {
                debug!("CLICK unparenting old popover");
                if old.1.parent().is_some() {
                    old.1.unparent();
                }
            }
            *tracker.borrow_mut() = None;
            sender_clone.input(DockItemInput::Click);
        });

        // Hover - show window list synchronously
        let motion = gtk::EventControllerMotion::new();
        let content_hover = self.content.clone();
        let popover_hover = self.popover.clone();
        let adapter = self.adapter.clone();
        let app_id = self.app_id.clone();
        let tracker = self.open_popover.clone();
        let root_hover = root.clone();

        motion.connect_enter(move |_controller, _x, _y| {
            let has = tracker.borrow().as_ref().is_some_and(|(tid, p)| tid == &app_id && p.parent().is_some());
            debug!(app_id, "ENTER has_tracker={has}");
            if has {
                return;
            }

            let windows = adapter
                .as_ref()
                .map(|a| a.get_windows(&app_id))
                .unwrap_or_default();

            if windows.is_empty() {
                debug!(app_id, "ENTER no_windows");
                return;
            }

            // Clear existing content
            while let Some(child) = content_hover.first_child() {
                content_hover.remove(&child);
            }

            for win in windows.iter() {
                let identifier = win.identifier.clone();
                let adapter = adapter.clone();
                let btn = gtk::Button::new();
                btn.add_css_class("dock-window-item");
                btn.set_label(&win.title);
                btn.set_hexpand(true);
                btn.set_halign(gtk::Align::Start);
                btn.set_valign(gtk::Align::Center);
                btn.connect_clicked(move |_| {
                    if let Some(ref a) = adapter {
                        a.focus_window(&identifier);
                    }
                });
                content_hover.append(&btn);
            }

            let has_parent = popover_hover.parent().is_some();
            debug!(app_id, has_parent, "ENTER has_parent");
            if has_parent {
                popover_hover.unparent();
            }
            popover_hover.set_parent(&root_hover);
            let parented = popover_hover.parent().is_some();
            debug!(app_id, parented, "ENTER after_set_parent");

            adapter::set_open_popover(&tracker, &app_id, &popover_hover);

            if !parented {
                debug!(app_id, "ENTER skipping popup (widget not parented)");
                return;
            }
            debug!(app_id, "ENTER calling popup");
            popover_hover.popup();
        });

        let content_leave = self.content.clone();
        let app_id_leave = self.app_id.clone();
        let tracker_leave = self.open_popover.clone();
        let popover_leave = self.popover.clone();

        motion.connect_leave(move |_| {
            debug!(app_id = %app_id_leave, "LEAVE start");
            if let Ok(current) = tracker_leave.try_borrow() {
                if current.as_ref().is_some_and(|(tid, _)| tid == &app_id_leave) {
                    debug!(app_id = %app_id_leave, "LEAVE same_app, returning early");
                    return;
                } else if let Some((tid, _)) = current.as_ref() {
                    debug!(app_id = %app_id_leave, other_tid = tid, "LEAVE diff_app_tracker");
                }
            }
            debug!(app_id = %app_id_leave, "LEAVE popdown");
            popover_leave.popdown();
            // Unparent so the popover can be reparented on next hover
            if popover_leave.parent().as_ref().is_some() {
                debug!(app_id = %app_id_leave, "LEAVE unparent");
                popover_leave.unparent();
            }
            while let Some(child) = content_leave.first_child() {
                content_leave.remove(&child);
            }
            debug!(app_id = %app_id_leave, "LEAVE clearing tracker");
            *tracker_leave.borrow_mut() = None;
        });

        root.add_controller(motion);

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            DockItemInput::Click => {
                let app_id = self.app_id.clone();
                let _ = sender.output((app_id, DockItemInput::Click));
            }
        }
    }
}

impl DockItem {
    fn update_model(&self, widgets: &<Self as relm4::factory::FactoryComponent>::Widgets) {
        widgets
            .icon_box
            .set_orientation(match self.settings.dock_position {
                DockPosition::Bottom => gtk::Orientation::Vertical,
                DockPosition::Left | DockPosition::Right => gtk::Orientation::Horizontal,
            });

        widgets
            .icon_image
            .set_pixel_size(self.settings.size.get() as i32);

        let icon_name = resolve_app_icon(&self.app_id);
        widgets.icon_image.set_icon_name(Some(&icon_name));

        widgets.indicator_revealer.set_reveal_child(self.is_running);

        self.update_indicator_alignment(widgets);

        if self.is_active {
            self._root.add_css_class("active");
        } else {
            self._root.remove_css_class("active");
        }

        if !self.is_pinned {
            self._root.add_css_class("unpinned");
        } else {
            self._root.remove_css_class("unpinned");
        }
    }

    fn update_indicator_alignment(
        &self,
        widgets: &<Self as relm4::factory::FactoryComponent>::Widgets,
    ) {
        let is_vertical = matches!(
            self.settings.dock_position,
            DockPosition::Left | DockPosition::Right
        );
        if let Some(dot) = widgets
            .indicator_revealer
            .child()
            .as_ref()
            .and_then(|c| c.downcast_ref::<gtk::Box>())
        {
            if is_vertical {
                dot.set_halign(gtk::Align::End);
                dot.set_valign(gtk::Align::Center);
                dot.remove_css_class("indicator-bottom");
                dot.add_css_class("indicator-horizontal");
            } else {
                dot.set_halign(gtk::Align::Center);
                dot.set_valign(gtk::Align::End);
                dot.remove_css_class("indicator-horizontal");
                dot.add_css_class("indicator-bottom");
            }
        }
    }
}
