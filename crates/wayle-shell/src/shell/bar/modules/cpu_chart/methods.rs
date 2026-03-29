use std::sync::Arc;

use gtk4::prelude::*;
use shared::rem_to_px;
use wayle_config::ConfigService;

use crate::shell::bar::modules::shared;

pub(super) fn setup_draw_func(
    drawing_area: &gtk4::DrawingArea,
    core_values: &std::rc::Rc<std::cell::Cell<Vec<f64>>>,
    config: &Arc<ConfigService>,
) {
    let values = core_values.clone();
    let config_clone = config.clone();

    drawing_area.set_draw_func(move |_area, cr: &gtk4::cairo::Context, _width, height| {
        let pixel_height = height as f64;

        let core_data = values.take();
        if core_data.is_empty() {
            values.set(core_data);
            return;
        }

        let full_config = config_clone.config();
        let cpuchart_config = &full_config.modules.cpuchart;

        let bar_width = cpuchart_config.bar_width.get() as f64;
        let bar_spacing = cpuchart_config.bar_gap.get() as f64;
        let bar_scale = full_config.bar.scale.get().value();
        let padding_rem = cpuchart_config.internal_padding.get().value();
        let horizontal_padding = rem_to_px(padding_rem, bar_scale);
        let direction = cpuchart_config.direction.get();
        let color = cpuchart_config.color.get();

        cr.translate(horizontal_padding, 0.0);

        let fill_color = shared::resolve_rgba(&color, &config_clone);

        wayle_widgets::primitives::barchart::draw_barchart(
            cr,
            &core_data,
            bar_width,
            bar_spacing,
            &wayle_widgets::primitives::chart::Params {
                fill_color,
                height: pixel_height,
                direction,
            },
        );

        values.set(core_data);
    });
}

pub(super) fn update_size(
    drawing_area: &gtk4::DrawingArea,
    num_cores: usize,
    config: &Arc<ConfigService>,
    is_vertical: bool,
) {
    let full_config = config.config();
    let cpuchart_config = &full_config.modules.cpuchart;
    let bar_scale = full_config.bar.scale.get().value();

    let bar_width = cpuchart_config.bar_width.get() as f64;
    let bar_spacing = cpuchart_config.bar_gap.get() as f64;
    let padding_rem = cpuchart_config.internal_padding.get().value();
    let horizontal_padding = rem_to_px(padding_rem, bar_scale);

    let total_width =
        (num_cores as f64 * (bar_width + bar_spacing)) - bar_spacing + (2.0 * horizontal_padding);

    if is_vertical {
        drawing_area.set_size_request(-1, total_width as i32);
    } else {
        drawing_area.set_size_request(total_width as i32, -1);
    }
}
