//! Theme preview swatch rendering. Uses a thread-local CSS provider to
//! avoid creating one provider per swatch.

use std::collections::BTreeSet;

use relm4::{
    gtk,
    gtk::{
        STYLE_PROVIDER_PRIORITY_USER, gdk::Display, prelude::*,
        style_context_add_provider_for_display,
    },
};
use tracing::warn;
use wayle_config::infrastructure::themes::Palette;

use super::{SWATCH_PROVIDER, SwatchStyles};

pub(super) fn build_swatches(palette: &Palette) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .valign(gtk::Align::Center)
        .build();
    container.add_css_class("theme-preset-swatches");

    for color in [
        &palette.bg,
        &palette.primary,
        &palette.red,
        &palette.yellow,
        &palette.green,
        &palette.blue,
    ] {
        let swatch = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .vexpand(false)
            .hexpand(false)
            .build();
        swatch.add_css_class("theme-preset-swatch");
        paint_swatch(&swatch, color);
        container.append(&swatch);
    }

    container
}

fn paint_swatch(widget: &gtk::Box, hex: &str) {
    let Some(class) = register_swatch_color(hex) else {
        return;
    };
    widget.add_css_class(&class);
}

fn register_swatch_color(hex: &str) -> Option<String> {
    let normalized = hex.trim_start_matches('#').to_lowercase();

    if normalized.len() != 6
        || !normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return None;
    }

    let class = format!("swatch-{normalized}");

    SWATCH_PROVIDER.with(|cell| {
        let mut slot = cell.borrow_mut();
        let styles = slot.get_or_insert_with(init_swatch_styles);

        if styles.registered_hexes.insert(normalized.clone()) {
            styles
                .provider
                .load_from_string(&rebuild_all_css(&styles.registered_hexes));
        }
    });

    Some(class)
}

fn rebuild_all_css(hexes: &BTreeSet<String>) -> String {
    let mut out = String::new();

    for hex in hexes {
        out.push_str(&format!(
            "box.theme-preset-swatch.swatch-{hex} {{ \
             background-color: #{hex}; background-image: none; }}\n"
        ));
    }

    out
}

fn init_swatch_styles() -> SwatchStyles {
    let provider = gtk::CssProvider::new();

    if let Some(display) = Display::default() {
        style_context_add_provider_for_display(&display, &provider, STYLE_PROVIDER_PRIORITY_USER);
    } else {
        warn!("no default display, theme swatch colors will not render");
    }

    SwatchStyles {
        provider,
        registered_hexes: BTreeSet::new(),
    }
}
