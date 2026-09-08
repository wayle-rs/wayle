//! CSS variable generation for bar styling.

use std::time::Duration;

use relm4::{ComponentSender, gtk};
use tokio::sync::mpsc;
use wayle_config::{
    SubscribeChanges,
    schemas::{bar::BorderLocation, styling::ThemeProvider},
};
use wayle_widgets::styling::{InlineStyling, resolve_color};

use super::{Bar, BarCmd};

const REM_BASE: f32 = 16.0;

/// GTK4 rendering is being weird. Ends up making icons blurry when icons are not
/// perfectly within the pixel boundary. So we make sure that they are with this
/// little workaround.
pub(super) fn rem_to_px_rounded(rem: f32, scale: f32) -> i32 {
    (rem * scale * REM_BASE).round() as i32
}

impl InlineStyling for Bar {
    type Sender = ComponentSender<Bar>;
    type Cmd = BarCmd;

    fn css_provider(&self) -> &gtk::CssProvider {
        &self.css_provider
    }

    fn spawn_style_watcher(&self, sender: &Self::Sender) {
        let config = self.services.config.config().clone();
        let bar = &config.bar;

        let (tx, mut rx) = mpsc::unbounded_channel();

        bar.scale.subscribe_changes(tx.clone());
        bar.inset_edge.subscribe_changes(tx.clone());
        bar.inset_ends.subscribe_changes(tx.clone());
        bar.padding.subscribe_changes(tx.clone());
        bar.padding_ends.subscribe_changes(tx.clone());
        bar.module_gap.subscribe_changes(tx.clone());
        bar.button_group_module_gap.subscribe_changes(tx.clone());
        bar.button_group_padding.subscribe_changes(tx.clone());
        bar.button_group_background.subscribe_changes(tx.clone());
        bar.button_group_opacity.subscribe_changes(tx.clone());
        bar.button_group_border_location
            .subscribe_changes(tx.clone());
        bar.button_group_border_width.subscribe_changes(tx.clone());
        bar.button_group_border_color.subscribe_changes(tx.clone());
        bar.button_group_rounding.subscribe_changes(tx.clone());
        bar.bg.subscribe_changes(tx.clone());
        bar.background_opacity.subscribe_changes(tx.clone());
        bar.button_opacity.subscribe_changes(tx.clone());
        bar.button_bg_opacity.subscribe_changes(tx.clone());
        bar.button_label_weight.subscribe_changes(tx.clone());
        bar.border_location.subscribe_changes(tx.clone());
        bar.border_width.subscribe_changes(tx.clone());
        bar.border_color.subscribe_changes(tx.clone());
        bar.shadow.subscribe_changes(tx);

        sender.command(move |out, shutdown| async move {
            const DEBOUNCE: Duration = Duration::from_millis(50);

            let shutdown_fut = shutdown.wait();
            tokio::pin!(shutdown_fut);

            loop {
                tokio::select! {
                    () = &mut shutdown_fut => break,
                    Some(()) = rx.recv() => {
                        loop {
                            tokio::select! {
                                () = &mut shutdown_fut => return,
                                Some(()) = rx.recv() => continue,
                                () = tokio::time::sleep(DEBOUNCE) => {
                                    let _ = out.send(BarCmd::StyleChanged);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    fn build_css(&self) -> String {
        let config = self.services.config.config();
        let bar = &config.bar;
        let styling = &config.styling;
        let is_wayle = matches!(styling.theme_provider.get(), ThemeProvider::Wayle);

        let bg = resolve_color(&bar.bg, is_wayle);
        let bg_opacity = bar.background_opacity.get().value();
        let button_opacity = f64::from(bar.button_opacity.get().value()) / 100.0;
        let button_bg_opacity = bar.button_bg_opacity.get().value();
        let label_weight = bar.button_label_weight.get().css_var();
        let border_color = resolve_color(&bar.border_color, is_wayle);
        let border_width = bar.border_width.get();
        let border_location = bar.border_location.get();

        let (border_top, border_bottom, border_left, border_right) = match border_location {
            BorderLocation::None => (0, 0, 0, 0),
            BorderLocation::Top => (border_width, 0, 0, 0),
            BorderLocation::Bottom => (0, border_width, 0, 0),
            BorderLocation::Left => (0, 0, border_width, 0),
            BorderLocation::Right => (0, 0, 0, border_width),
            BorderLocation::All => (border_width, border_width, border_width, border_width),
        };

        let scale = bar.scale.get().value();
        let inset_edge_px = rem_to_px_rounded(bar.inset_edge.get().value(), scale);
        let inset_ends_px = rem_to_px_rounded(bar.inset_ends.get().value(), scale);
        let padding_px = rem_to_px_rounded(bar.padding.get().value(), scale);
        let padding_ends_px = rem_to_px_rounded(bar.padding_ends.get().value(), scale);
        let module_gap_px = rem_to_px_rounded(bar.module_gap.get().value(), scale);
        let group_module_gap_px =
            rem_to_px_rounded(bar.button_group_module_gap.get().value(), scale);
        let group_padding_px =
            rem_to_px_rounded(bar.button_group_padding.get().value() * 0.25, scale);
        let group_bg = resolve_color(&bar.button_group_background, is_wayle);
        let group_opacity = bar.button_group_opacity.get().value();
        let group_border_color = resolve_color(&bar.button_group_border_color, is_wayle);
        let group_border_width = bar.button_group_border_width.get();
        let group_border_location = bar.button_group_border_location.get();

        let (group_border_top, group_border_bottom, group_border_left, group_border_right) =
            match group_border_location {
                BorderLocation::None => (0, 0, 0, 0),
                BorderLocation::Top => (group_border_width, 0, 0, 0),
                BorderLocation::Bottom => (0, group_border_width, 0, 0),
                BorderLocation::Left => (0, 0, group_border_width, 0),
                BorderLocation::Right => (0, 0, 0, group_border_width),
                BorderLocation::All => (
                    group_border_width,
                    group_border_width,
                    group_border_width,
                    group_border_width,
                ),
            };

        let location = bar.location.get();
        let shadow_preset = bar.shadow.get();
        let shadow = shadow_preset.css_shadow(location);
        let shadow_margin = shadow_preset.opposite_margin();

        format!(
            ".bar {{ \
            --bar-scale: {scale}; \
            --bar-bg: {bg}; \
            --bar-opacity: {bg_opacity}%; \
            --bar-border-color: {border_color}; \
            --bar-border-top: {border_top}; \
            --bar-border-bottom: {border_bottom}; \
            --bar-border-left: {border_left}; \
            --bar-border-right: {border_right}; \
            --bar-inset-edge-px: {inset_edge_px}; \
            --bar-inset-ends-px: {inset_ends_px}; \
            --bar-padding-px: {padding_px}; \
            --bar-padding-ends-px: {padding_ends_px}; \
            --bar-module-gap-px: {module_gap_px}; \
            --bar-button-opacity: {button_opacity}; \
            --bar-button-bg-opacity: {button_bg_opacity}%; \
            --bar-btn-label-weight: var({label_weight}); \
            --bar-group-module-gap-px: {group_module_gap_px}; \
            --bar-group-padding-px: {group_padding_px}; \
            --bar-group-bg: {group_bg}; \
            --bar-group-opacity: {group_opacity}%; \
            --bar-group-border-color: {group_border_color}; \
            --bar-group-border-top: {group_border_top}; \
            --bar-group-border-bottom: {group_border_bottom}; \
            --bar-group-border-left: {group_border_left}; \
            --bar-group-border-right: {group_border_right}; \
            --bar-shadow: {shadow}; \
            --bar-shadow-margin: {shadow_margin}; \
            }}"
        )
    }
}
