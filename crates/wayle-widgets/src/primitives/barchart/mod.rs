//! Generic barchart rendering primitives using Cairo.
//!
//! Provides reusable components for rendering vertical barcharts with
//! configurable width, spacing, direction, and colors.

use gtk4::cairo;
use wayle_config::schemas::barchart::BarDirection;

/// Minimum height for a bar in pixels to ensure visibility.
pub const MIN_BAR_HEIGHT: f64 = 2.0;

/// Parameters for rendering a barchart.
pub struct BarchartParams {
    /// Width of each bar in pixels.
    pub bar_width: f64,
    /// Spacing between bars in pixels.
    pub bar_spacing: f64,
    /// Fill color for the bars.
    pub fill_color: Rgba,
}

/// RGBA color with components normalized to 0.0-1.0.
pub struct Rgba {
    /// Red component (0.0-1.0).
    pub red: f64,
    /// Green component (0.0-1.0).
    pub green: f64,
    /// Blue component (0.0-1.0).
    pub blue: f64,
    /// Alpha/opacity component (0.0-1.0).
    pub alpha: f64,
}

/// Draws a barchart visualization using Cairo.
///
/// Each value in the `values` slice represents a bar's amplitude (0.0-1.0),
/// which is scaled to the canvas height. Bars are drawn with the specified
/// width, spacing, color, and direction.
pub fn draw_barchart(
    cr: &cairo::Context,
    values: &[f64],
    canvas_height: f64,
    direction: BarDirection,
    params: &BarchartParams,
) {
    apply_color(cr, params);

    let bar_stride = params.bar_width + params.bar_spacing;

    for (bar_idx, &amplitude) in values.iter().enumerate() {
        let x = bar_idx as f64 * bar_stride;
        let bar_height = (amplitude * canvas_height).clamp(MIN_BAR_HEIGHT, canvas_height);

        fill_bar_rect(
            cr,
            x,
            bar_height,
            canvas_height,
            direction,
            params.bar_width,
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

fn apply_color(cr: &cairo::Context, params: &BarchartParams) {
    let color = &params.fill_color;
    cr.set_source_rgba(color.red, color.green, color.blue, color.alpha);
}

fn bar_origin_y(direction: BarDirection, bar_height: f64, canvas_height: f64) -> f64 {
    match direction {
        BarDirection::Normal => canvas_height - bar_height,
        BarDirection::Reverse => 0.0,
        BarDirection::Mirror => (canvas_height - bar_height) / 2.0,
    }
}

fn fill_bar_rect(
    cr: &cairo::Context,
    x: f64,
    bar_height: f64,
    canvas_height: f64,
    direction: BarDirection,
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
