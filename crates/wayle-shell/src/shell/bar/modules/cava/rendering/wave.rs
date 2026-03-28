use gtk4::cairo;
use wayle_config::schemas::barchart::BarDirection;

use super::{RenderParams, apply_color};

const MIN_WAVE_HEIGHT: f64 = 2.0;

pub(crate) fn draw_wave(
    cr: &cairo::Context,
    values: &[f64],
    canvas_width: f64,
    canvas_height: f64,
    direction: BarDirection,
    params: &RenderParams,
) {
    if values.is_empty() {
        return;
    }

    apply_color(cr, params);

    let num_points = values.len();
    let point_spacing = if num_points > 1 {
        canvas_width / (num_points - 1) as f64
    } else {
        canvas_width
    };

    let min_amplitude = MIN_WAVE_HEIGHT / canvas_height;

    let amplitude_to_y = |amplitude: f64| -> f64 {
        let clamped_amplitude = amplitude.max(min_amplitude);
        match direction {
            BarDirection::Normal => canvas_height * (1.0 - clamped_amplitude),
            BarDirection::Reverse => canvas_height * clamped_amplitude,
            BarDirection::Mirror => canvas_height * (1.0 - clamped_amplitude) / 2.0,
        }
    };

    trace_wave_curve(cr, values, point_spacing, &amplitude_to_y);
    close_wave_path(
        cr,
        values,
        canvas_width,
        canvas_height,
        point_spacing,
        min_amplitude,
        direction,
    );

    cr.close_path();
    let _ = cr.fill();
}

fn trace_wave_curve(
    cr: &cairo::Context,
    values: &[f64],
    point_spacing: f64,
    amplitude_to_y: &dyn Fn(f64) -> f64,
) {
    cr.move_to(0.0, amplitude_to_y(values[0]));

    for point_idx in 1..values.len() {
        let point_x = point_idx as f64 * point_spacing;
        let prev_point_x = (point_idx - 1) as f64 * point_spacing;
        let bezier_control_x = (prev_point_x + point_x) / 2.0;

        cr.curve_to(
            bezier_control_x,
            amplitude_to_y(values[point_idx - 1]),
            bezier_control_x,
            amplitude_to_y(values[point_idx]),
            point_x,
            amplitude_to_y(values[point_idx]),
        );
    }
}

fn close_wave_path(
    cr: &cairo::Context,
    values: &[f64],
    canvas_width: f64,
    canvas_height: f64,
    point_spacing: f64,
    min_amplitude: f64,
    direction: BarDirection,
) {
    match direction {
        BarDirection::Normal => {
            cr.line_to(canvas_width, canvas_height);
            cr.line_to(0.0, canvas_height);
        }
        BarDirection::Reverse => {
            cr.line_to(canvas_width, 0.0);
            cr.line_to(0.0, 0.0);
        }
        BarDirection::Mirror => {
            trace_mirror_bottom(cr, values, point_spacing, min_amplitude, canvas_height);
        }
    }
}

fn trace_mirror_bottom(
    cr: &cairo::Context,
    values: &[f64],
    point_spacing: f64,
    min_amplitude: f64,
    canvas_height: f64,
) {
    let num_points = values.len();
    let center = canvas_height / 2.0;

    for point_idx in (0..num_points).rev() {
        let point_x = point_idx as f64 * point_spacing;
        let clamped_amplitude = values[point_idx].max(min_amplitude);
        let mirror_y = center + clamped_amplitude * canvas_height / 2.0;

        if point_idx == num_points - 1 {
            cr.line_to(point_x, mirror_y);
            continue;
        }

        let next_point_x = (point_idx + 1) as f64 * point_spacing;
        let bezier_control_x = (point_x + next_point_x) / 2.0;
        let next_clamped_amplitude = values[point_idx + 1].max(min_amplitude);
        let next_mirror_y = center + next_clamped_amplitude * canvas_height / 2.0;

        cr.curve_to(
            bezier_control_x,
            next_mirror_y,
            bezier_control_x,
            mirror_y,
            point_x,
            mirror_y,
        );
    }
}
