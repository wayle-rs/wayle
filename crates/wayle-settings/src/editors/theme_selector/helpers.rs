//! Theme preview swatch rendering and list population.
//!
//! Swatches reuse a thread-local CSS provider to avoid creating one provider
//! per color.

use std::collections::BTreeSet;

use relm4::{
    gtk,
    gtk::{
        STYLE_PROVIDER_PRIORITY_USER, gdk::Display, prelude::*,
        style_context_add_provider_for_display,
    },
    prelude::*,
};
use tracing::warn;
use wayle_config::{
    ConfigProperty,
    infrastructure::themes::Palette,
    schemas::styling::{HexColor, PaletteConfig, ThemeEntry},
};

use super::{SWATCH_PROVIDER, SwatchStyles, ThemeSelectorControl, ThemeSelectorMsg};

pub(super) fn populate_list(
    list_box: &gtk::ListBox,
    themes: &[ThemeEntry],
    sender: &ComponentSender<ThemeSelectorControl>,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    for theme in themes {
        let row = build_theme_row(theme, sender);
        list_box.append(&row);
    }
}

fn build_theme_row(
    theme: &ThemeEntry,
    sender: &ComponentSender<ThemeSelectorControl>,
) -> gtk::Button {
    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    content.add_css_class("theme-preset-row");

    let swatches = build_swatches(&theme.palette);
    content.append(&swatches);

    let label = gtk::Label::builder()
        .label(&theme.name)
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    label.add_css_class("theme-preset-name");
    content.append(&label);

    let button = gtk::Button::new();
    button.set_child(Some(&content));
    button.add_css_class("theme-preset-entry");
    button.set_cursor_from_name(Some("pointer"));

    let theme_name = theme.name.clone();
    let sender = sender.clone();
    button.connect_clicked(move |_| {
        sender.input(ThemeSelectorMsg::Apply(theme_name.clone()));
    });

    button
}

pub(super) fn apply_palette(target: &PaletteConfig, source: &Palette) {
    set_if_valid(&target.bg, &source.bg);
    set_if_valid(&target.surface, &source.surface);
    set_if_valid(&target.elevated, &source.elevated);
    set_if_valid(&target.fg, &source.fg);
    set_if_valid(&target.fg_muted, &source.fg_muted);
    set_if_valid(&target.primary, &source.primary);
    set_if_valid(&target.red, &source.red);
    set_if_valid(&target.yellow, &source.yellow);
    set_if_valid(&target.green, &source.green);
    set_if_valid(&target.blue, &source.blue);
}

fn set_if_valid(property: &ConfigProperty<HexColor>, hex_str: &str) {
    if let Ok(hex) = HexColor::new(hex_str) {
        property.set(hex);
    }
}

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
