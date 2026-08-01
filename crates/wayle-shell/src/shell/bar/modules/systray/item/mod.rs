mod helpers;
mod menu;
mod methods;
mod watchers;

use std::{cell::Cell, rc::Rc, sync::Arc};

use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use wayle_config::ConfigService;
use wayle_systray::{
    core::item::TrayItem,
    types::{Coordinates, menu::MenuItem},
};

use crate::shell::bar::dropdowns::{DismissFn, OpenSurfaceCoordinator, SECONDARY_OPENER_CSS_CLASS};

pub(super) struct SystrayItemInit {
    pub(super) item: Arc<TrayItem>,
    pub(super) config: Arc<ConfigService>,
    pub(super) coordinator: Rc<OpenSurfaceCoordinator>,
}

pub(super) struct SystrayItem {
    item: Arc<TrayItem>,
    config: Arc<ConfigService>,
    button: Option<gtk::Button>,
    icon: Option<gtk::Image>,
    icon_signature: Option<IconSignature>,
    icon_color_provider: Option<gtk::CssProvider>,
    icon_color_provider_attached: bool,
    /// The cached cascade, pre-built off the click path when the layout arrives (see
    /// `rebuild_cached_menu`) so a click only shows it. Reused across open/close
    /// cycles and rebuilt only when the layout changes.
    menu: Option<menu::TrayMenu>,
    /// The [`MenuItem`] tree the cached `menu` was built from, so a DBusMenu update
    /// that doesn't actually change the layout can skip the (surface-recreating)
    /// rebuild. Set/cleared together with `menu` and `menu_reg`. See
    /// `rebuild_cached_menu`.
    displayed_menu: Option<MenuItem>,
    coordinator: Rc<OpenSurfaceCoordinator>,
    /// The cached menu's dismiss closure and a `registered` flag ("is the coordinator's
    /// open surface"). Created with `menu` and reused across shows: each present sets
    /// the flag, and the coordinator registration is released exactly once — the
    /// popover's `connect_closed` or an explicit teardown, whichever flips it first.
    menu_reg: Option<(DismissFn, Rc<Cell<bool>>)>,
    cancel_token: CancellationToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum IconSignature {
    File(String),
    Named(String),
    Pixmap(u64),
    Fallback,
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub(super) enum SystrayItemMsg {
    LeftClick,
    RightClick,
    MiddleClick,
    /// Toggle this item's menu from the CLI (`wayle systray toggle <id>`), routed
    /// through the same path as a right-click.
    ToggleMenu,
    /// Open this item's menu from the CLI (`wayle systray open <id>`): open if
    /// closed, no-op if already open.
    OpenMenu,
    MenuUpdated,
    IconUpdated,
}

#[derive(Debug)]
pub(super) enum SystrayItemOutput {}

#[relm4::factory(pub(super))]
impl FactoryComponent for SystrayItem {
    type Init = SystrayItemInit;
    type Input = SystrayItemMsg;
    type Output = SystrayItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            // `dropdown-opener-secondary` marks this as a right-click opener so the
            // bar's automatic press-dismiss skips it on a right-click (see
            // `handle_bar_click`); otherwise the bar closes the menu on press and the
            // tray gesture re-opens it. It MUST live in `set_css_classes` here (not a
            // later `add_css_class`), because `set_css_classes` replaces the whole
            // class list when the view is built and would wipe a separately-added mark.
            set_css_classes: &["systray-item", SECONDARY_OPENER_CSS_CLASS],
            set_cursor_from_name: Some("pointer"),

            #[name = "icon"]
            gtk::Image {},
        }
    }

    fn init_model(
        init: Self::Init,
        _index: &relm4::factory::DynamicIndex,
        _sender: relm4::prelude::FactorySender<Self>,
    ) -> Self {
        Self {
            item: init.item,
            config: init.config,
            button: None,
            icon: None,
            icon_signature: None,
            icon_color_provider: None,
            icon_color_provider_attached: false,
            menu: None,
            displayed_menu: None,
            coordinator: init.coordinator,
            menu_reg: None,
            cancel_token: CancellationToken::new(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &relm4::factory::DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: relm4::prelude::FactorySender<Self>,
    ) -> Self::Widgets {
        let item_id = self.item.id.get();
        root.set_widget_name(&item_id);
        debug!(item_id = %item_id, "init_widgets: setting up button");

        self.button = Some(root.clone());

        root.connect_clicked({
            let sender = sender.clone();
            move |_| {
                sender.input(SystrayItemMsg::LeftClick);
            }
        });

        let right_click = gtk::GestureClick::builder().button(3).build();
        let middle_click = gtk::GestureClick::builder().button(2).build();

        // Open on PRESS, not release: the scrim and the bar's automatic dismiss both
        // act on press, so a right-click whose press and release route to different
        // surfaces (possible under the stationary-pointer focus deferral right after
        // a menu maps) would otherwise let the scrim dismiss on press and the tray
        // re-open on release. Firing on press keeps the whole click on one surface.
        right_click.connect_pressed({
            let sender = sender.clone();
            move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender.input(SystrayItemMsg::RightClick);
            }
        });

        middle_click.connect_pressed({
            let sender = sender.clone();
            move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender.input(SystrayItemMsg::MiddleClick);
            }
        });

        root.add_controller(right_click);
        root.add_controller(middle_click);

        watchers::spawn_menu_watcher(&sender, &self.item, self.cancel_token.clone());
        watchers::spawn_icon_watcher(&sender, &self.item, self.cancel_token.clone());

        let widgets = view_output!();

        self.icon = Some(widgets.icon.clone());
        self.update_icon(&widgets.icon);

        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: relm4::prelude::FactorySender<Self>) {
        match msg {
            SystrayItemMsg::LeftClick => {
                let item = self.item.clone();
                let item_is_menu = item.item_is_menu.get();
                tokio::spawn(async move {
                    let result = if item_is_menu {
                        item.context_menu(Coordinates::new(0, 0)).await
                    } else {
                        item.activate(Coordinates::new(0, 0)).await
                    };
                    if let Err(error) = result {
                        warn!(
                            id = %item.id.get(),
                            bus_name = %item.bus_name.get(),
                            error = %error,
                            "systray activate failed"
                        );
                    }
                });
            }
            SystrayItemMsg::RightClick | SystrayItemMsg::ToggleMenu => {
                self.request_menu_show(true);
            }
            SystrayItemMsg::OpenMenu => {
                self.request_menu_show(false);
            }
            SystrayItemMsg::MiddleClick => {
                let item = self.item.clone();
                tokio::spawn(async move {
                    if let Err(error) = item.secondary_activate(Coordinates::new(0, 0)).await {
                        warn!(
                            id = %item.id.get(),
                            bus_name = %item.bus_name.get(),
                            error = %error,
                            "systray secondary_activate failed"
                        );
                    }
                });
            }
            SystrayItemMsg::MenuUpdated => {
                self.rebuild_cached_menu();
            }
            SystrayItemMsg::IconUpdated => {
                if let Some(icon) = self.icon.clone() {
                    self.update_icon(&icon);
                }
            }
        }
    }
}

impl SystrayItem {
    /// The stable SNI id used to address this item from the CLI.
    pub(super) fn tray_id(&self) -> String {
        self.item.id.get()
    }
}

impl Drop for SystrayItem {
    fn drop(&mut self) {
        self.cancel_token.cancel();
        self.clear_menu_registration();
        if let Some(menu) = self.menu.take() {
            menu.teardown();
        }
    }
}
