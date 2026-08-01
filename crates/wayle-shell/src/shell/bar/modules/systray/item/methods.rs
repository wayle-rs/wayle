use std::{cell::Cell, path::Path, rc::Rc};

#[allow(deprecated)]
use gtk4::prelude::StyleContextExt;
use relm4::gtk::{self, prelude::*};
use tracing::debug;
use wayle_systray::types::{Coordinates, menu::MenuItem};

use super::{
    IconSignature, SystrayItem, menu,
    helpers::{
        create_texture_from_pixmap, find_icon_in_theme_path, hash_pixmaps,
        load_icon_from_theme_path, load_scaled_texture_from_file, select_best_pixmap,
    },
};
use crate::shell::{
    bar::{dropdowns::DismissFn, modules::systray::helpers::find_override},
    helpers::COMPONENT_CSS_PRIORITY,
};

/// A freshly built cascade plus the bookkeeping to show it and later release it:
/// the tree it was built from (for the no-op-skip), its stable dismiss closure, and
/// the `registered` flag its one-time `closed` handler flips.
struct BuiltMenu {
    menu: menu::TrayMenu,
    tree: MenuItem,
    dismiss: DismissFn,
    registered: Rc<Cell<bool>>,
}

impl SystrayItem {
    /// Toggle/open this item's menu on click. `toggle`: right-click / `systray
    /// toggle` (open if closed, close if open); `false` is `systray open`
    /// (open-if-closed, no-op if already open).
    ///
    /// The open/close decision goes through the [`OpenSurfaceCoordinator`], keyed on
    /// the tray button as the anchor — the same authority the dropdowns use — rather
    /// than a transient `menu.is_visible()` check. That check could go stale between
    /// the press that closed the menu (via the scrim) and the release that acted on
    /// it, so a second right-click sometimes re-opened instead of staying closed.
    /// The cascade is normally pre-built off the click path (by the menu watcher),
    /// so opening only *presents* it; a click before the first layout arrives
    /// cold-builds it (see [`show_menu`]).
    pub(super) fn request_menu_show(&mut self, toggle: bool) {
        let Some(button) = self.button.clone() else {
            return;
        };
        let anchor: gtk::Widget = button.upcast();
        // Clone the Rc so the open closure borrows only `self`, not `self.coordinator`.
        let coordinator = self.coordinator.clone();

        if toggle {
            coordinator.toggle(&anchor, || self.present_or_show());
        } else {
            coordinator.open_only(&anchor, || self.present_or_show());
        }
    }

    /// The coordinator's "open" action for this item: present the pre-built cascade
    /// (or cold-build it), then kick off a background `AboutToShow` refresh — which
    /// reconciles the menu in place if the app returns a changed layout. Runs only
    /// when the coordinator decides to open (never on a toggle-off).
    fn present_or_show(&mut self) {
        if self.menu.is_some() {
            self.present_cached();
        } else {
            // No cache yet (menu layout not received) — build and show now.
            self.show_menu();
        }

        let item = self.item.clone();
        tokio::spawn(async move {
            if let Err(error) = item.refresh_menu().await {
                debug!(error = %error, "AboutToShow not supported");
            }
        });
    }

    /// Apply a layout change from the menu watcher, OFF the click path. Skips when
    /// the layout is unchanged. If the cascade is already built, it is PATCHED IN
    /// PLACE via [`menu::TrayMenu::reconcile`] — reusing every popover, button, and
    /// submenu column — whether the menu is hidden or visible (a visible reconcile
    /// is the point: an AboutToShow/LayoutUpdated change updates the open menu with
    /// no flicker and no surface churn). If it isn't built yet, build it once and
    /// cache it hidden. If the layout goes empty, drop the cache.
    pub(super) fn rebuild_cached_menu(&mut self) {
        let new_tree = self.item.menu.get();
        if new_tree == self.displayed_menu {
            return;
        }

        let has_content = new_tree
            .as_ref()
            .is_some_and(|root| !root.children.is_empty());

        if self.menu.is_some() {
            if has_content {
                if let (Some(menu), Some(root)) = (self.menu.as_ref(), new_tree.as_ref()) {
                    menu.reconcile(root);
                }
                self.displayed_menu = new_tree;
            } else {
                // Layout emptied/unavailable: drop the cache (dismiss it if open).
                let visible = self.menu.as_ref().is_some_and(menu::TrayMenu::is_visible);
                self.drop_cache(visible);
            }
        } else if has_content
            && let Some(built) = self.build_cascade()
        {
            // First layout: build the persistent cascade once, hidden.
            self.install(built);
        }
    }

    /// Build the cascade for the current layout and install its one-time `closed`
    /// handler, without showing it. `None` when there is no usable layout.
    fn build_cascade(&self) -> Option<BuiltMenu> {
        let root_menu = self.item.menu.get()?;
        if root_menu.children.is_empty() {
            return None;
        }
        let parent = self.button.clone()?;

        // Bar scale sizes the root menu's bar-gap offset (to match the dropdown
        // panels); styling scale sizes the submenu flush offset (to match the panel's
        // `space-xs` contents padding, which uses `--global-scale`).
        let config = self.config.config();
        let scale = config.bar.scale.get().value();
        let styling_scale = config.styling.scale.get().value();
        let menu = menu::build(&self.item, &root_menu, parent.upcast_ref(), scale, styling_scale);
        let dismiss = menu.dismiss_handle();
        // `registered` = "this menu is the coordinator's open surface, so closing it
        // must notify the coordinator". Set true on each present; flipped false
        // exactly once by whichever fires first — the popover's `closed` (a popdown)
        // or an explicit teardown (`clear_menu_registration`, since `teardown`
        // unparents without popping down). Installed once here, so reusing the cached
        // popover across open/close cycles never stacks handlers.
        let registered = Rc::new(Cell::new(false));
        menu.root_popover().connect_closed({
            let coordinator = self.coordinator.clone();
            let dismiss = dismiss.clone();
            let registered = registered.clone();
            move |_| {
                if registered.replace(false) {
                    coordinator.notify_closed(&dismiss);
                }
            }
        });

        Some(BuiltMenu {
            menu,
            tree: root_menu,
            dismiss,
            registered,
        })
    }

    /// Present the already-built cached cascade: register it as the open surface
    /// (closing any other, showing the scrim) and pop it up. No widget construction.
    fn present_cached(&mut self) {
        let Some(menu) = self.menu.as_ref() else {
            return;
        };
        let Some((dismiss, registered)) = self.menu_reg.as_ref() else {
            return;
        };
        let Some(parent) = self.button.clone() else {
            return;
        };

        registered.set(true);
        let anchor = parent.upcast_ref::<gtk::Widget>().downgrade();
        // Register as the open surface FIRST — this closes any other open surface and
        // establishes the scrim + bar->Overlay stacking (coordinator.open -> sync_scrim
        // -> scrim.show raises the bar above the scrim) — THEN pop the menu up so it
        // maps as a child of the already-raised bar and stacks ABOVE the scrim. This
        // matches the dropdown open ceremony (registry.rs `open_on`); doing popup()
        // first left the fresh popup stacked under the scrim on a switch (scrim over
        // the bar), which then swallowed the next click. The needs-motion swap bug is
        // still avoided because the scrim stays mapped across the swap (coordinator
        // show-before-close), so a stationary pointer always has a live surface.
        self.coordinator
            .open(dismiss.clone(), Some(menu.key_handler()), Some(anchor));
        menu.popup();
    }

    /// Cold path: a click arrived before the watcher cached the cascade (the app
    /// hadn't sent a layout yet). Build it once, cache it, and present it — or fall
    /// back to the app's own context menu when there's still no layout. Subsequent
    /// opens hit the cache and go straight through `present_cached`.
    fn show_menu(&mut self) {
        match self.build_cascade() {
            Some(built) => {
                self.install(built);
                self.present_cached();
            }
            None => {
                debug!("no menu data, falling back");
                self.spawn_context_menu_fallback();
            }
        }
    }

    /// Cache a freshly built cascade (hidden). Only called when there is no existing
    /// cache — the persistent surface is built exactly once and thereafter updated
    /// in place by [`rebuild_cached_menu`], never rebuilt.
    fn install(&mut self, built: BuiltMenu) {
        debug_assert!(
            self.menu.is_none(),
            "install replaces no surface; updates go through reconcile"
        );
        let BuiltMenu {
            menu,
            tree,
            dismiss,
            registered,
        } = built;

        self.menu = Some(menu);
        self.menu_reg = Some((dismiss, registered));
        self.displayed_menu = Some(tree);
    }

    /// Drop the cached cascade (the layout went empty/unavailable), dismissing it
    /// first if it was open so a stale menu never lingers on screen.
    fn drop_cache(&mut self, visible: bool) {
        if visible {
            self.coordinator.dismiss_current();
        }
        self.clear_menu_registration();
        if let Some(old_menu) = self.menu.take() {
            old_menu.teardown();
        }
        self.menu_reg = None;
        self.displayed_menu = None;
    }

    /// Release the cached menu's coordinator registration (which hides the scrim) if
    /// it is currently the open surface and its popover's `closed` hasn't already
    /// done so — `teardown` unparents without popping down, so `closed` may not fire.
    pub(super) fn clear_menu_registration(&mut self) {
        if let Some((dismiss, registered)) = self.menu_reg.as_ref()
            && registered.replace(false)
        {
            self.coordinator.notify_closed(dismiss);
        }
    }

    fn spawn_context_menu_fallback(&self) {
        let item = self.item.clone();
        tokio::spawn(async move {
            let _ = item.context_menu(Coordinates::new(0, 0)).await;
        });
    }

    pub(super) fn update_icon(&mut self, image: &gtk::Image) {
        let overrides = self.config.config().modules.systray.overrides.get();
        let override_match = find_override(&self.item, &overrides);

        let icon_name = override_match
            .and_then(|entry| entry.icon.clone())
            .or_else(|| self.item.icon_name.get());

        let icon_signature = self.icon_signature(icon_name.as_deref());

        if self.icon_signature.as_ref() != Some(&icon_signature) {
            self.apply_icon(image, icon_name.as_deref());
            self.icon_signature = Some(icon_signature);
        }

        if let Some(color) = override_match.and_then(|entry| entry.color.clone()) {
            self.apply_icon_color(image, &color.to_css());
        } else {
            self.clear_icon_color(image);
        }
    }

    fn icon_signature(&self, icon_name: Option<&str>) -> IconSignature {
        if let Some(name) = icon_name {
            let theme_path = self.item.icon_theme_path.get();

            if let Some(file_path) = theme_path
                .as_deref()
                .and_then(|path| find_icon_in_theme_path(path, name))
            {
                return IconSignature::File(file_path);
            }

            if Path::new(name).is_file() {
                return IconSignature::File(name.to_owned());
            }

            return IconSignature::Named(name.to_owned());
        }

        let pixmaps = self.item.icon_pixmap.get();
        if pixmaps.is_empty() {
            return IconSignature::Fallback;
        }

        IconSignature::Pixmap(hash_pixmaps(&pixmaps))
    }

    fn apply_icon_color(&mut self, image: &gtk::Image, css_color: &str) {
        let provider = self
            .icon_color_provider
            .get_or_insert_with(gtk::CssProvider::new);

        let css = format!("image {{ color: {css_color}; }}");
        provider.load_from_string(&css);

        if !self.icon_color_provider_attached {
            #[allow(deprecated)]
            image
                .style_context()
                .add_provider(provider, COMPONENT_CSS_PRIORITY);
            self.icon_color_provider_attached = true;
        }
    }

    fn clear_icon_color(&mut self, image: &gtk::Image) {
        if self.icon_color_provider_attached
            && let Some(provider) = self.icon_color_provider.as_ref()
        {
            #[allow(deprecated)]
            image.style_context().remove_provider(provider);
            self.icon_color_provider_attached = false;
        }
    }

    fn apply_icon(&self, image: &gtk::Image, icon_name: Option<&str>) {
        image.clear();

        if let Some(name) = icon_name {
            let theme_path = self.item.icon_theme_path.get();
            if let Some(texture) = theme_path
                .as_deref()
                .and_then(|path| load_icon_from_theme_path(path, name))
            {
                image.set_paintable(Some(&texture));
                return;
            }

            if let Some(texture) = load_scaled_texture_from_file(name) {
                image.set_paintable(Some(&texture));
                return;
            }

            image.set_icon_name(Some(name));
            return;
        }

        let pixmaps = self.item.icon_pixmap.get();
        if let Some(texture) = select_best_pixmap(&pixmaps).and_then(create_texture_from_pixmap) {
            image.set_paintable(Some(&texture));
            return;
        }

        image.set_icon_name(Some("application-x-executable-symbolic"));
    }
}
