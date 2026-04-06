use std::path::Path;

#[allow(deprecated)]
use gtk4::prelude::StyleContextExt;
use gtk4::{gdk, gio, glib::idle_add_local_once};
use relm4::{
    gtk::{self, prelude::*},
    prelude::*,
};
use tracing::debug;
use wayle_systray::{
    adapters::gtk4::{Adapter, TrayMenuModel},
    types::Coordinates,
};

use super::{
    IconSignature, SystrayItem, SystrayItemMsg,
    helpers::{
        create_texture_from_pixmap, find_icon_in_theme_path, hash_pixmaps,
        load_icon_from_theme_path, load_scaled_texture_from_file, select_best_pixmap,
    },
};
use crate::shell::bar::modules::systray::helpers::find_override;

impl SystrayItem {
    pub(super) fn request_menu_show(&self, sender: &FactorySender<Self>) {
        if let Some(popover) = self.popover.as_ref()
            && popover.is_visible()
        {
            debug!(item_id = %self.item.id.get(), "hiding popover");
            popover.popdown();
            return;
        }

        let item = self.item.clone();
        let sender = sender.clone();

        tokio::spawn(async move {
            if let Err(error) = item.refresh_menu().await {
                debug!(error = %error, "AboutToShow not supported");
            }

            sender.input(SystrayItemMsg::ShowMenu);
        });
    }

    pub(super) fn toggle_menu(&mut self) {
        if let Some(popover) = self.popover.as_ref()
            && popover.is_visible()
        {
            debug!(item_id = %self.item.id.get(), "hiding popover");
            popover.popdown();
            return;
        }

        self.show_menu();
    }

    #[allow(clippy::cognitive_complexity)]
    fn show_menu(&mut self) {
        let item_id = self.item.id.get();
        debug!(item_id = %item_id, title = %self.item.title.get(), "show_menu called");

        let menu_data = self.item.menu.get();
        let Some(root_menu) = menu_data else {
            debug!("no menu data, falling back");
            self.spawn_context_menu_fallback();
            return;
        };

        if root_menu.children.is_empty() {
            debug!("empty menu, falling back");
            self.spawn_context_menu_fallback();
            return;
        }

        let model = Adapter::build_model(&self.item);
        debug!(
            item_id = %item_id,
            menu_n_items = model.menu.n_items(),
            accelerators = model.accelerators.len(),
            "built menu model"
        );

        let popover = self.ensure_popover(&model.menu);
        self.apply_menu_model(&popover, model);
        popover.popup();
    }

    pub(super) fn ensure_popover(&mut self, menu: &gio::Menu) -> gtk::PopoverMenu {
        if let Some(popover) = self.popover.clone() {
            return popover;
        }

        let popover = gtk::PopoverMenu::from_model_full(menu, gtk::PopoverMenuFlags::NESTED);
        popover.add_css_class("systray-menu");
        popover.set_has_arrow(false);

        popover.connect_map(|popover| {
            override_model_button_layout(popover.upcast_ref());
        });

        if let Some(parent) = self.button.as_ref() {
            popover.set_parent(parent);
        }

        self.popover = Some(popover.clone());
        popover
    }

    pub(super) fn apply_menu_model(&mut self, popover: &gtk::PopoverMenu, model: TrayMenuModel) {
        self.clear_accelerators();
        popover.set_menu_model(Some(&model.menu));
        popover.insert_action_group("app", Some(&model.actions));
        self.register_accelerators(popover, &model.accelerators);
        self.action_group = Some(model.actions);
    }

    fn register_accelerators(
        &mut self,
        popover: &gtk::PopoverMenu,
        accelerators: &[(String, String)],
    ) {
        let Some(app) = popover
            .root()
            .and_then(|root| root.downcast::<gtk::Window>().ok())
            .and_then(|window| window.application())
        else {
            return;
        };

        for (action_name, accel) in accelerators {
            app.set_accels_for_action(action_name, &[accel.as_str()]);
            self.registered_accels.push(action_name.clone());
        }
    }

    pub(super) fn clear_accelerators(&mut self) {
        let accels: Vec<String> = self.registered_accels.drain(..).collect();

        let Some(popover) = self.popover.as_ref() else {
            return;
        };

        let Some(app) = popover
            .root()
            .and_then(|root| root.downcast::<gtk::Window>().ok())
            .and_then(|window| window.application())
        else {
            return;
        };

        for action_name in &accels {
            app.set_accels_for_action(action_name, &[]);
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
                .add_provider(provider, gtk::STYLE_PROVIDER_PRIORITY_USER + 1);
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
        image.set_icon_name(None);
        image.set_paintable(None::<&gdk::Texture>);

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

    pub(super) fn rebuild_menu_if_visible(&mut self) {
        let Some(popover) = self.popover.clone() else {
            return;
        };

        if !popover.is_visible() {
            return;
        }

        let model = Adapter::build_model(&self.item);
        self.apply_menu_model(&popover, model);

        idle_add_local_once(move || override_model_button_layout(popover.upcast_ref()));
    }
}

/// GTK4's `GtkModelButton` hides icons when a label is present and reserves
/// left margin for check/radio indicators via a shared size group, even on
/// items that don't have one. This walks the popover tree and undoes both:
/// icons with content get forced visible, and empty indicator boxes get hidden.
fn override_model_button_layout(widget: &gtk::Widget) {
    if widget.css_name() == "modelbutton" {
        force_icon_visible(widget);
        hide_empty_indicator_box(widget);
        return;
    }

    let mut child = widget.first_child();
    while let Some(current) = child {
        override_model_button_layout(current.upcast_ref());
        child = current.next_sibling();
    }
}

/// GTK4 hides the icon on model buttons that have a label. If the icon
/// actually has content (a theme icon, GIcon, or paintable), force it visible.
fn force_icon_visible(button: &gtk::Widget) {
    let mut child = button.first_child();

    while let Some(current) = child {
        if let Some(image) = current.downcast_ref::<gtk::Image>() {
            let has_content = image.icon_name().is_some()
                || image.gicon().is_some()
                || image.paintable().is_some();

            if has_content {
                image.set_visible(true);
            }
        }

        child = current.next_sibling();
    }
}

/// Every model button has a `box` child that holds check/radio indicators.
/// GTK puts all of these boxes in a shared size group so they align, which
/// means items without indicators still reserve that space on the left.
/// Hide the box when it's empty to reclaim the margin.
fn hide_empty_indicator_box(button: &gtk::Widget) {
    let mut child = button.first_child();

    while let Some(current) = child {
        if current.css_name() == "box" && !contains_toggle_indicator(&current) {
            current.set_visible(false);
        }

        child = current.next_sibling();
    }
}

fn contains_toggle_indicator(indicator_box: &gtk::Widget) -> bool {
    let mut child = indicator_box.first_child();

    while let Some(current) = child {
        let name = current.css_name();
        if name == "check" || name == "radio" {
            return true;
        }
        child = current.next_sibling();
    }

    false
}
