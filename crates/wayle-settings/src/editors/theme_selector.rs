//! "Apply theme preset" action button. Opens a popover listing available
//! themes; clicking one overwrites the current palette colors with that
//! theme's values. The button has no persistent selection state.

use std::{cell::RefCell, collections::BTreeSet};

use gtk4::{
    STYLE_PROVIDER_PRIORITY_USER, gdk::Display, prelude::*, style_context_add_provider_for_display,
};
use relm4::prelude::*;
use tracing::warn;
use wayle_config::{
    ConfigProperty,
    infrastructure::themes::Palette,
    schemas::styling::{HexColor, PaletteConfig, ThemeEntry},
};
use wayle_i18n::t;

use super::{ControlOutput, spawn_property_watcher};

thread_local! {
    static SWATCH_PROVIDER: RefCell<Option<SwatchStyles>> = const { RefCell::new(None) };
}

struct SwatchStyles {
    provider: gtk4::CssProvider,
    registered_hexes: BTreeSet<String>,
}

pub(crate) struct ThemeSelectorControl {
    available: ConfigProperty<Vec<ThemeEntry>>,
    palette: PaletteConfig,
    popover: gtk4::Popover,
    list_box: gtk4::ListBox,
}

pub(crate) struct ThemeSelectorInit {
    pub available: ConfigProperty<Vec<ThemeEntry>>,
    pub palette: PaletteConfig,
}

#[derive(Debug)]
pub(crate) enum ThemeSelectorMsg {
    Apply(String),
    RebuildList,
}

impl SimpleComponent for ThemeSelectorControl {
    type Init = ThemeSelectorInit;
    type Input = ThemeSelectorMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .hexpand(false)
            .valign(gtk4::Align::Center)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let button_content = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .build();

        let button_icon = gtk4::Image::from_icon_name("ld-palette-symbolic");
        button_icon.add_css_class("theme-preset-button-icon");
        button_content.append(&button_icon);

        let button_label = gtk4::Label::new(Some(&t("settings-theme-preset-apply")));
        button_label.add_css_class("theme-preset-button-label");
        button_content.append(&button_label);

        let button_chevron = gtk4::Image::from_icon_name("ld-chevron-down-symbolic");
        button_chevron.add_css_class("theme-preset-button-chevron");
        button_content.append(&button_chevron);

        let button = gtk4::Button::builder().child(&button_content).build();
        button.add_css_class("theme-preset-button");
        button.set_cursor_from_name(Some("pointer"));

        let list_box = gtk4::ListBox::new();
        list_box.add_css_class("theme-preset-list");
        list_box.set_selection_mode(gtk4::SelectionMode::None);

        let scrolled = gtk4::ScrolledWindow::builder()
            .child(&list_box)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .propagate_natural_height(true)
            .build();
        scrolled.add_css_class("theme-preset-scroll");

        let popover = gtk4::Popover::builder()
            .child(&scrolled)
            .has_arrow(false)
            .build();
        popover.add_css_class("theme-preset-popover");
        popover.set_parent(&button);

        let popover_ref = popover.clone();
        button.connect_clicked(move |_| popover_ref.popup());

        populate_list(&list_box, &init.available.get(), &sender);

        let input_sender = sender.input_sender().clone();
        spawn_property_watcher(&init.available, move || {
            let _ = input_sender.send(ThemeSelectorMsg::RebuildList);
        });

        let popover_cleanup = popover.clone();
        button.connect_destroy(move |_| popover_cleanup.unparent());

        root.append(&button);

        let model = Self {
            available: init.available,
            palette: init.palette,
            popover,
            list_box,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ThemeSelectorMsg::Apply(name) => {
                let themes = self.available.get();

                let Some(theme) = themes.iter().find(|entry| entry.name == name) else {
                    return;
                };

                apply_palette(&self.palette, &theme.palette);
                self.popover.popdown();
                let _ = sender.output(ControlOutput::ValueChanged);
            }

            ThemeSelectorMsg::RebuildList => {
                populate_list(&self.list_box, &self.available.get(), &sender);
            }
        }
    }
}

fn populate_list(
    list_box: &gtk4::ListBox,
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
) -> gtk4::Button {
    let content = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .build();
    content.add_css_class("theme-preset-row");

    let swatches = build_swatches(&theme.palette);
    content.append(&swatches);

    let label = gtk4::Label::builder()
        .label(&theme.name)
        .halign(gtk4::Align::Start)
        .hexpand(true)
        .build();
    label.add_css_class("theme-preset-name");
    content.append(&label);

    let button = gtk4::Button::new();
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

fn build_swatches(palette: &Palette) -> gtk4::Box {
    let container = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Horizontal)
        .valign(gtk4::Align::Center)
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
        let swatch = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Horizontal)
            .valign(gtk4::Align::Center)
            .halign(gtk4::Align::Center)
            .vexpand(false)
            .hexpand(false)
            .build();
        swatch.add_css_class("theme-preset-swatch");
        paint_swatch(&swatch, color);
        container.append(&swatch);
    }

    container
}

fn paint_swatch(widget: &gtk4::Box, hex: &str) {
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
            "box.theme-preset-swatch.swatch-{hex} {{ background-color: #{hex}; background-image: none; }}\n"
        ));
    }

    out
}

fn init_swatch_styles() -> SwatchStyles {
    let provider = gtk4::CssProvider::new();

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
