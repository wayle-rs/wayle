//! Generic barchart rendering primitives using Cairo.
//!
//! Provides reusable components for rendering vertical barcharts with
//! configurable width, spacing, direction, and colors.

use gtk4::cairo;
use wayle_config::schemas::types::chart::Direction;

use crate::primitives::chart::Params;

/// Minimum height for a bar in pixels to ensure visibility.
pub const MIN_BAR_HEIGHT: f64 = 2.0;

/// Parameters for rendering a barchart.
pub struct BarchartParams {
    /// Width of each bar in pixels.
    pub bar_width: f64,
    /// Spacing between bars in pixels.
    pub bar_spacing: f64,
    /// Rendering parameters (color, etc.).
    pub chart_params: super::chart::Params,
}

/// Draws a barchart visualization using Cairo.
///
/// Each value in the `values` slice represents a bar's amplitude (0.0-1.0),
/// which is scaled to the canvas height. Bars are drawn with the specified
/// width, spacing, color, and direction.
pub fn draw_barchart(
    cr: &cairo::Context,
    values: &[f64],
    bar_width: f64,
    bar_spacing: f64,
    params: &Params,
) {
    apply_color(cr, params);

    let bar_stride = bar_width + bar_spacing;

    for (bar_idx, &amplitude) in values.iter().enumerate() {
        let x = bar_idx as f64 * bar_stride;
        let bar_height = (amplitude * params.height).clamp(MIN_BAR_HEIGHT, params.height);

        fill_bar_rect(
            cr,
            x,
            bar_height,
            params.height,
            params.direction,
            bar_width,
        );
        let _ = cr.fill();
    }
}
/// Calculates the total widget length needed to display the barchart.
///
/// Takes into account the number of bars, their width, spacing between them,
/// and padding on both ends.
pub fn calculate_widget_length(bars: u16, bar_width: u32, bar_gap: u32, padding: f64) -> i32 {
    let bar_count = f64::from(bars);
    let gap_count = (bar_count - 1.0).max(0.0);
    let bar_space = bar_count * f64::from(bar_width);
    let gap_space = gap_count * f64::from(bar_gap);
    let pad_space = padding * 2.0;

    let total = bar_space + gap_space + pad_space;
    total.round().max(1.0) as i32
}

fn apply_color(cr: &cairo::Context, params: &Params) {
    let color = &params.fill_color;
    cr.set_source_rgba(color.red, color.green, color.blue, color.alpha);
}

fn bar_origin_y(direction: Direction, bar_height: f64, canvas_height: f64) -> f64 {
    match direction {
        Direction::Normal => canvas_height - bar_height,
        Direction::Reverse => 0.0,
        Direction::Mirror => (canvas_height - bar_height) / 2.0,
    }
}

fn fill_bar_rect(
    cr: &cairo::Context,
    x: f64,
    bar_height: f64,
    canvas_height: f64,
    direction: Direction,
    bar_width: f64,
) {
    let y = bar_origin_y(direction, bar_height, canvas_height);
    cr.rectangle(x, y, bar_width, bar_height);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_length_single_bar() {
        assert_eq!(calculate_widget_length(1, 3, 1, 0.0), 3);
    }

    #[test]
    fn widget_length_multiple_bars() {
        assert_eq!(calculate_widget_length(20, 3, 1, 0.0), 79);
    }

    #[test]
    fn widget_length_zero_gap() {
        assert_eq!(calculate_widget_length(10, 5, 0, 0.0), 50);
    }

    #[test]
    fn widget_length_with_padding() {
        assert_eq!(calculate_widget_length(20, 3, 1, 8.0), 95);
    }
}
