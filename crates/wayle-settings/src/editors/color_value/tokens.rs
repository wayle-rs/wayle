//! ColorValue dropdown item catalog: Auto, Transparent, Custom entries
//! plus grouped CSS token entries with headers.

use wayle_config::schemas::styling::{ColorValue, CssToken};
use wayle_i18n::t;

pub(super) const HEADER_ID: &str = "__header";
pub(super) const AUTO_ID: &str = "__auto";
pub(super) const TRANSPARENT_ID: &str = "__transparent";
pub(super) const CUSTOM_ID: &str = "__custom";

pub(super) struct ColorItem {
    pub(crate) id: &'static str,
    pub(crate) label: String,
    pub(crate) value: ColorValue,
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
