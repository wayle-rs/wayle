mod peaks;
mod wave;

use gtk4::cairo;
use wayle_config::schemas::types::chart::Direction;
use wayle_widgets::primitives::{barchart::MIN_BAR_HEIGHT, chart::Params};

pub(super) use self::{peaks::draw_peak_bars, wave::draw_wave};

pub(super) fn apply_color(cr: &cairo::Context, params: &Params) {
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
