use gtk4::cairo;
use wayle_config::schemas::barchart::BarDirection;

use super::{MIN_BAR_HEIGHT, RenderParams, apply_color, fill_bar_rect};

const PEAK_CAP_HEIGHT: f64 = 2.0;
const PEAK_GRAVITY: f64 = 0.015;

/// Per-bar peak amplitude for decay tracking.
pub(crate) type PeakState = Vec<f64>;

pub(crate) fn draw_peak_bars(
    cr: &cairo::Context,
    values: &[f64],
    peaks: &mut PeakState,
    canvas_height: f64,
    direction: BarDirection,
    params: &RenderParams,
) {
    apply_color(cr, params);

    let bar_stride = params.bar_width + params.bar_spacing;

    peaks.resize(values.len(), 0.0);

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

        update_peak(&mut peaks[bar_idx], amplitude);

        let peak_height = peaks[bar_idx] * canvas_height;
        draw_peak_cap(
            cr,
            x,
            peak_height,
            bar_height,
            canvas_height,
            direction,
            params.bar_width,
        );
    }
}

fn update_peak(peak: &mut f64, current_value: f64) {
    if current_value >= *peak {
        *peak = current_value;
    } else {
        *peak = (*peak - PEAK_GRAVITY).max(0.0);
    }
}

fn draw_peak_cap(
    cr: &cairo::Context,
    x: f64,
    peak_height: f64,
    bar_height: f64,
    canvas_height: f64,
    direction: BarDirection,
    bar_width: f64,
) {
    if peak_height <= bar_height {
        return;
    }

    let cap_height = PEAK_CAP_HEIGHT.min(canvas_height);

    match direction {
        BarDirection::Normal => {
            cr.rectangle(
                x,
                canvas_height - peak_height - cap_height,
                bar_width,
                cap_height,
            );
        }
        BarDirection::Reverse => {
            cr.rectangle(x, peak_height, bar_width, cap_height);
        }
        BarDirection::Mirror => {
            let peak_half = peak_height / 2.0;
            let center = canvas_height / 2.0;

            cr.rectangle(x, center - peak_half - cap_height, bar_width, cap_height);
            let _ = cr.fill();
            cr.rectangle(x, center + peak_half, bar_width, cap_height);
        }
    }

    let _ = cr.fill();
}
