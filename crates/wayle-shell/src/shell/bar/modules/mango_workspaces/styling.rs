//! Live CSS provider applied to each `MangoWorkspaces` component instance.

use std::sync::Arc;

use relm4::gtk;
use wayle_config::{ConfigService, schemas::styling::ThemeProvider};
use wayle_widgets::{prelude::BarSettings, styling::resolve_color};

use super::helpers;

const REM_BASE: f32 = 16.0;
const ICON_BASE_REM: f32 = 1.3;
const LABEL_BASE_REM: f32 = 1.1;

fn rem_to_px_rounded(rem: f32, scale: f32) -> i32 {
    (rem * scale * REM_BASE).round() as i32
}

/// CSS class for a tag-map key, or `None` when the key is not a tag index.
fn tag_map_css_class(key: &str) -> Option<String> {
    let index: u32 = key.parse().ok()?;
    Some(helpers::tag_css_class(index))
}

pub(super) fn apply_styling(
    provider: &gtk::CssProvider,
    config_service: &Arc<ConfigService>,
    settings: &BarSettings,
) {
    let config = config_service.config();
    let tags_config = &config.modules.mango_workspaces;
    let is_wayle_theme = matches!(config.styling.theme_provider.get(), ThemeProvider::Wayle);

    let active_color = resolve_color(&tags_config.active_color, is_wayle_theme);
    let occupied_color = resolve_color(&tags_config.occupied_color, is_wayle_theme);
    let empty_color = resolve_color(&tags_config.empty_color, is_wayle_theme);
    let container_bg_color = resolve_color(&tags_config.container_bg_color, is_wayle_theme);
    let border_color = resolve_color(&tags_config.border_color, is_wayle_theme);
    let border_width = settings.border_width.get();

    let bar_scale = config.bar.scale.get().value();
    let icon_scale = tags_config.icon_size.get().value();
    let label_scale = tags_config.label_size.get().value();
    let is_vertical = settings.is_vertical.get();

    let icon_size_px = rem_to_px_rounded(ICON_BASE_REM * icon_scale, bar_scale);
    let label_size_px = rem_to_px_rounded(LABEL_BASE_REM * label_scale, bar_scale);
    let tag_padding_px = rem_to_px_rounded(tags_config.tag_padding.get().value(), bar_scale);

    let (margin_vertical_px, margin_horizontal_px) = if is_vertical {
        (tag_padding_px, 0)
    } else {
        (0, tag_padding_px)
    };

    let mut css = format!(
        ".workspaces.mango {{ \
            --ws-active-color: {active_color}; \
            --ws-occupied-color: {occupied_color}; \
            --ws-empty-color: {empty_color}; \
            --ws-container-bg-color: {container_bg_color}; \
            --ws-border-color: {border_color}; \
            --ws-border-width: {border_width}px; \
            --ws-icon-size-px: {icon_size_px}; \
            --ws-label-size-px: {label_size_px}; \
            --ws-margin-vertical-px: {margin_vertical_px}; \
            --ws-margin-horizontal-px: {margin_horizontal_px}; \
        }}"
    );

    for (index, style) in &tags_config.tag_map.get() {
        let Some(color) = style.color.as_ref() else {
            continue;
        };
        let Some(selector_class) = tag_map_css_class(index) else {
            continue;
        };
        let color_css = color.to_css();
        css.push_str(&format!(
            ".workspaces.mango .workspace.{selector_class} {{ --ws-override-color: {color_css}; }}"
        ));
    }

    provider.load_from_string(&css);
}
