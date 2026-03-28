mod peaks;
mod wave;

use gtk4::cairo;
use wayle_config::schemas::barchart::BarDirection;
use wayle_widgets::primitives::barchart::{BarchartParams, MIN_BAR_HEIGHT};

pub(super) use self::{peaks::draw_peak_bars, wave::draw_wave};

// Use BarchartParams directly instead of wrapper type
pub(super) type RenderParams = BarchartParams;

fn apply_color(cr: &cairo::Context, params: &RenderParams) {
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
