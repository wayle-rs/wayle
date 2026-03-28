use std::{cell::Cell, rc::Rc, sync::Arc};

use gtk::{glib::Propagation, prelude::*};
use relm4::prelude::*;
use wayle_config::{ConfigService, schemas::modules::CavaStyle};
use wayle_widgets::{
    primitives::barchart::calculate_widget_length, primitives::barchart::draw_barchart,
};

use super::{CavaModule, helpers, messages::CavaMsg, rendering};
use crate::shell::bar::modules::shared;

impl CavaModule {
    pub(super) fn attach_click_gesture(widget: &gtk::Box, sender: &ComponentSender<Self>) {
        let click = gtk::GestureClick::new();
        click.set_button(0);

        let input_sender = sender.input_sender().clone();
        click.connect_pressed(move |gesture, _n_press, _x, _y| {
            let msg = match gesture.current_button() {
                1 => CavaMsg::LeftClick,
                2 => CavaMsg::MiddleClick,
                3 => CavaMsg::RightClick,
                _ => return,
            };
            input_sender.emit(msg);
        });

        widget.add_controller(click);
    }

    pub(super) fn attach_scroll_controller(
        widget: &gtk::Box,
        sender: &ComponentSender<Self>,
        sensitivity: f64,
    ) {
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let threshold = 0.5 / sensitivity.max(0.1);

        let input_sender = sender.input_sender().clone();
        scroll.connect_scroll(move |_controller, _dx, dy| {
            if dy.abs() < threshold {
                return Propagation::Proceed;
            }
            let msg = if dy < 0.0 {
                CavaMsg::ScrollUp
            } else {
                CavaMsg::ScrollDown
            };
            input_sender.emit(msg);
            Propagation::Stop
        });

        widget.add_controller(scroll);
    }

    pub(super) fn setup_draw_func(
        drawing_area: &gtk::DrawingArea,
        frame_data: &Rc<Cell<Vec<f64>>>,
        is_vertical: bool,
        config: &Arc<ConfigService>,
    ) {
        let frame_values = frame_data.clone();
        let full_config = config.config();
        let cava_config = &full_config.modules.cava;

        let style = cava_config.style.get();
        let direction = cava_config.direction.get();
        let bar_width = cava_config.bar_width.get() as f64;
        let bar_spacing = cava_config.bar_gap.get() as f64;
        let bar_scale = full_config.bar.scale.get().value();
        let fill_color = shared::resolve_rgba(&cava_config.color.get(), config);
        let padding_rem = cava_config.internal_padding.get().value();
        let horizontal_padding = helpers::rem_to_px(padding_rem, bar_scale);

        let render_params = rendering::RenderParams {
            bar_width,
            bar_spacing,
            fill_color,
        };

        let peak_state = Cell::new(Vec::<f64>::new());

        drawing_area.set_draw_func(move |_area, cr, width, height| {
            let pixel_width = width as f64;
            let pixel_height = height as f64;

            let values = frame_values.take();
            if values.is_empty() {
                frame_values.set(values);
                return;
            }

            let (canvas_width, canvas_height) = if is_vertical {
                (pixel_height, pixel_width)
            } else {
                (pixel_width, pixel_height)
            };
            let content_width = (canvas_width - horizontal_padding * 2.0).max(0.0);

            if is_vertical {
                cr.translate(0.0, pixel_height);
                cr.rotate(-std::f64::consts::FRAC_PI_2);
            }

            cr.translate(horizontal_padding, 0.0);

            match style {
                CavaStyle::Bars => {
                    draw_barchart(cr, &values, canvas_height, direction, &render_params);
                }
                CavaStyle::Wave => {
                    rendering::draw_wave(
                        cr,
                        &values,
                        content_width,
                        canvas_height,
                        direction,
                        &render_params,
                    );
                }
                CavaStyle::Peaks => {
                    let mut peaks = peak_state.take();
                    rendering::draw_peak_bars(
                        cr,
                        &values,
                        &mut peaks,
                        canvas_height,
                        direction,
                        &render_params,
                    );
                    peak_state.set(peaks);
                }
            }

            frame_values.set(values);
        });
    }

    pub(super) fn update_size(&self) {
        let full_config = self.config.config();
        let cava_config = &full_config.modules.cava;
        let bars = cava_config.bars.get().value();
        let bar_width = cava_config.bar_width.get();
        let bar_gap = cava_config.bar_gap.get();
        let bar_scale = full_config.bar.scale.get().value();
        let padding_rem = cava_config.internal_padding.get().value();
        let padding_px = helpers::rem_to_px(padding_rem, bar_scale);
        let length = calculate_widget_length(bars, bar_width, bar_gap, padding_px);

        if self.is_vertical {
            self.drawing_area.set_size_request(-1, length);
            self.drawing_area.set_hexpand(true);
            self.drawing_area.set_vexpand(false);
        } else {
            self.drawing_area.set_size_request(length, -1);
            self.drawing_area.set_vexpand(true);
            self.drawing_area.set_hexpand(false);
        }
    }
}
