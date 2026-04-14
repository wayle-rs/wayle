//! Window chrome: close-button overlay and sidebar width clamp.

use relm4::gtk::{self, prelude::*};
use wayle_config::Config;
use wayle_i18n::t;

const MAX_SIDEBAR_REM: f64 = 25.0;
const BASE_PX_PER_REM: f64 = 16.0;

pub(super) fn build_content_overlay(stack: &gtk::Stack, window: &gtk::Window) -> gtk::Overlay {
    let close_button = gtk::Button::from_icon_name("ld-x-symbolic");
    close_button.add_css_class("settings-close");
    close_button.set_cursor_from_name(Some("pointer"));
    close_button.set_valign(gtk::Align::Start);
    close_button.set_halign(gtk::Align::End);
    close_button.set_tooltip_text(Some(&t("settings-close")));

    let window_ref = window.clone();
    close_button.connect_clicked(move |_| window_ref.close());

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(stack));
    overlay.add_overlay(&close_button);
    overlay
}

pub(super) fn setup_paned_clamp(paned: &gtk::Paned, config: &Config) {
    let scale_property = config.styling.scale.clone();

    paned.connect_position_notify(move |paned| {
        let scale = scale_property.get().value() as f64;
        let max_width = (MAX_SIDEBAR_REM * BASE_PX_PER_REM * scale).round() as i32;

        if paned.position() > max_width {
            paned.set_position(max_width);
        }
    });
}
