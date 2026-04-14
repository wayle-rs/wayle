//! Theme application and list rebuild handlers for `ThemeSelectorControl`.

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    infrastructure::themes::Palette,
    schemas::styling::{HexColor, PaletteConfig, ThemeEntry},
};

use super::{ThemeSelectorControl, ThemeSelectorMsg, helpers::build_swatches};

impl ThemeSelectorControl {
    pub(super) fn on_apply(&mut self, name: String) {
        let themes = self.available.get();

        let Some(theme) = themes.iter().find(|entry| entry.name == name) else {
            return;
        };

        apply_palette(&self.palette, &theme.palette);
        self.popover.popdown();
    }

    pub(super) fn on_rebuild_list(&mut self, sender: &ComponentSender<Self>) {
        populate_list(&self.list_box, &self.available.get(), sender);
    }
}

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

fn apply_palette(target: &PaletteConfig, source: &Palette) {
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
