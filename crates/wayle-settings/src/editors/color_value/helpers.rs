//! Color dropdown data: item construction, token metadata, factory setup,
//! and color conversion utilities.

use relm4::{
    gtk,
    gtk::{gdk, prelude::*},
};
use wayle_config::schemas::styling::{ColorValue, CssToken};
use wayle_i18n::t;

pub(crate) const HEADER_ID: &str = "__header";
pub(crate) const AUTO_ID: &str = "__auto";
pub(crate) const TRANSPARENT_ID: &str = "__transparent";
pub(crate) const CUSTOM_ID: &str = "__custom";

pub(crate) struct ColorItem {
    pub id: &'static str,
    pub label: String,
    pub value: ColorValue,
}

pub(super) fn hex_to_rgba(hex: &str) -> gdk::RGBA {
    gdk::RGBA::parse(hex).unwrap_or(gdk::RGBA::new(0.0, 0.0, 0.0, 1.0))
}

pub(super) fn rgba_to_hex(rgba: &gdk::RGBA) -> String {
    let red = (rgba.red() * 255.0).round() as u8;
    let green = (rgba.green() * 255.0).round() as u8;
    let blue = (rgba.blue() * 255.0).round() as u8;

    format!("#{red:02x}{green:02x}{blue:02x}")
}

pub(super) fn find_index(items: &[ColorItem], value: &ColorValue) -> u32 {
    match value {
        ColorValue::Auto => find_by_id(items, AUTO_ID),
        ColorValue::Transparent => find_by_id(items, TRANSPARENT_ID),
        ColorValue::Custom(_) => find_by_id(items, CUSTOM_ID),
        ColorValue::Token(token) => {
            let target = token.as_str().strip_prefix("--").unwrap_or(token.as_str());
            find_by_id(items, target)
        }
    }
}

fn find_by_id(items: &[ColorItem], target: &str) -> u32 {
    items
        .iter()
        .position(|color_item| color_item.id == target)
        .unwrap_or(0) as u32
}

pub(super) fn setup_dropdown_factory(dropdown: &gtk::DropDown, items: &[ColorItem]) {
    let item_data: Vec<(&'static str, String)> = items
        .iter()
        .map(|color_item| (color_item.id, color_item.label.clone()))
        .collect();

    let factory = gtk::SignalListItemFactory::new();

    factory.connect_setup(|_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        list_item.set_child(Some(&build_row_template()));
    });

    let data = item_data;

    factory.connect_bind(move |_factory, list_item| {
        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
            return;
        };

        let position = list_item.position() as usize;

        let Some((id, label_text)) = data.get(position) else {
            return;
        };

        bind_row(list_item, id, label_text);
    });

    dropdown.set_factory(Some(&factory));
}

fn build_row_template() -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();
    row.add_css_class("color-value-row");

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("color-value-dot");
    dot.set_vexpand(false);
    dot.set_hexpand(false);
    dot.set_valign(gtk::Align::Center);
    dot.set_halign(gtk::Align::Center);

    let label = gtk::Label::builder()
        .halign(gtk::Align::Start)
        .hexpand(true)
        .build();
    label.add_css_class("color-value-label");

    row.append(&dot);
    row.append(&label);

    row
}

fn bind_row(list_item: &gtk::ListItem, id: &str, label_text: &str) {
    let Some(child) = list_item.child() else {
        return;
    };

    let Some(row) = child.downcast_ref::<gtk::Box>() else {
        return;
    };

    let Some(dot) = row.first_child() else {
        return;
    };

    let Some(label_widget) = dot.next_sibling() else {
        return;
    };

    let Some(label) = label_widget.downcast_ref::<gtk::Label>() else {
        return;
    };

    if id == HEADER_ID {
        dot.set_visible(false);
        label.set_label(label_text);
        label.set_css_classes(&["color-value-group-header"]);
        row.set_css_classes(&["color-value-header-row"]);
        return;
    }

    dot.set_visible(true);
    row.set_css_classes(&["color-value-row"]);
    label.set_css_classes(&["color-value-label"]);
    label.set_label(label_text);

    dot.set_css_classes(&["color-value-dot"]);

    match id {
        AUTO_ID => dot.set_visible(false),
        TRANSPARENT_ID => dot.add_css_class("transparent"),
        CUSTOM_ID => dot.add_css_class("custom"),
        _ => dot.add_css_class(&format!("token-{id}")),
    }
}

fn token_meta(token: CssToken) -> (&'static str, &'static str, &'static str) {
    match token {
        CssToken::BgBase => ("bg-base", "Base", "Backgrounds"),
        CssToken::BgSurface => ("bg-surface", "Surface", "Backgrounds"),
        CssToken::BgSurfaceElevated => ("bg-surface-elevated", "Surface Elevated", "Backgrounds"),
        CssToken::BgElevated => ("bg-elevated", "Elevated", "Backgrounds"),
        CssToken::BgOverlay => ("bg-overlay", "Overlay", "Backgrounds"),
        CssToken::BgHover => ("bg-hover", "Hover", "Backgrounds"),
        CssToken::BgActive => ("bg-active", "Active", "Backgrounds"),
        CssToken::BgSelected => ("bg-selected", "Selected", "Backgrounds"),

        CssToken::FgDefault => ("fg-default", "Default", "Foregrounds"),
        CssToken::FgMuted => ("fg-muted", "Muted", "Foregrounds"),
        CssToken::FgSubtle => ("fg-subtle", "Subtle", "Foregrounds"),
        CssToken::FgOnAccent => ("fg-on-accent", "On Accent", "Foregrounds"),

        CssToken::Accent => ("accent", "Accent", "Accent"),
        CssToken::AccentSubtle => ("accent-subtle", "Accent Subtle", "Accent"),
        CssToken::AccentHover => ("accent-hover", "Accent Hover", "Accent"),

        CssToken::StatusError => ("status-error", "Error", "Status"),
        CssToken::StatusWarning => ("status-warning", "Warning", "Status"),
        CssToken::StatusSuccess => ("status-success", "Success", "Status"),
        CssToken::StatusInfo => ("status-info", "Info", "Status"),
        CssToken::StatusErrorSubtle => ("status-error-subtle", "Error Subtle", "Status"),
        CssToken::StatusWarningSubtle => ("status-warning-subtle", "Warning Subtle", "Status"),
        CssToken::StatusSuccessSubtle => ("status-success-subtle", "Success Subtle", "Status"),
        CssToken::StatusInfoSubtle => ("status-info-subtle", "Info Subtle", "Status"),
        CssToken::StatusErrorHover => ("status-error-hover", "Error Hover", "Status"),

        CssToken::Red => ("red", "Red", "Semantic"),
        CssToken::Yellow => ("yellow", "Yellow", "Semantic"),
        CssToken::Green => ("green", "Green", "Semantic"),
        CssToken::Blue => ("blue", "Blue", "Semantic"),

        CssToken::BorderSubtle => ("border-subtle", "Subtle", "Borders"),
        CssToken::BorderDefault => ("border-default", "Default", "Borders"),
        CssToken::BorderStrong => ("border-strong", "Strong", "Borders"),
        CssToken::BorderAccent => ("border-accent", "Accent", "Borders"),
        CssToken::BorderError => ("border-error", "Error", "Borders"),
    }
}

const ALL_TOKENS: &[CssToken] = &[
    CssToken::BgBase,
    CssToken::BgSurface,
    CssToken::BgSurfaceElevated,
    CssToken::BgElevated,
    CssToken::BgOverlay,
    CssToken::BgHover,
    CssToken::BgActive,
    CssToken::BgSelected,
    CssToken::FgDefault,
    CssToken::FgMuted,
    CssToken::FgSubtle,
    CssToken::FgOnAccent,
    CssToken::Accent,
    CssToken::AccentSubtle,
    CssToken::AccentHover,
    CssToken::StatusError,
    CssToken::StatusWarning,
    CssToken::StatusSuccess,
    CssToken::StatusInfo,
    CssToken::StatusErrorSubtle,
    CssToken::StatusWarningSubtle,
    CssToken::StatusSuccessSubtle,
    CssToken::StatusInfoSubtle,
    CssToken::StatusErrorHover,
    CssToken::Red,
    CssToken::Yellow,
    CssToken::Green,
    CssToken::Blue,
    CssToken::BorderSubtle,
    CssToken::BorderDefault,
    CssToken::BorderStrong,
    CssToken::BorderAccent,
    CssToken::BorderError,
];

pub(super) fn build_items() -> Vec<ColorItem> {
    let mut items = vec![
        ColorItem {
            id: AUTO_ID,
            label: t("settings-color-auto"),
            value: ColorValue::Auto,
        },
        ColorItem {
            id: TRANSPARENT_ID,
            label: t("settings-color-transparent"),
            value: ColorValue::Transparent,
        },
        ColorItem {
            id: CUSTOM_ID,
            label: t("settings-color-custom"),
            value: ColorValue::Auto,
        },
    ];

    let mut last_group = "";

    for &token in ALL_TOKENS {
        let (id, label, group) = token_meta(token);

        if group != last_group {
            items.push(ColorItem {
                id: HEADER_ID,
                label: group.to_string(),
                value: ColorValue::Auto,
            });
            last_group = group;
        }

        items.push(ColorItem {
            id,
            label: label.to_string(),
            value: ColorValue::Token(token),
        });
    }

    items
}
