mod factory;
mod messages;
mod watchers;

use std::{cell::Cell, rc::Rc, sync::Arc};

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{CpuChartCmd, CpuChartInit, CpuChartMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct CpuChartModule {
    bar_button: Controller<BarButton>,
    drawing_area: gtk4::DrawingArea,
    core_values: Rc<Cell<Vec<f64>>>,
    config: Arc<ConfigService>,
    is_vertical: bool,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for CpuChartModule {
    type Init = CpuChartInit;
    type Input = CpuChartMsg;
    type Output = ();
    type CommandOutput = CpuChartCmd;

    view! {
        gtk::Box {
            add_css_class: "cpu-chart",

            #[local_ref]
            bar_button -> gtk::MenuButton {},
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let config = init.config.config();
        let cpuchart_config = &config.modules.cpuchart;

        let cpu_data = init.sysinfo.cpu.get();
        let num_cores = cpu_data.cores.len();
        let core_values: Rc<Cell<Vec<f64>>> = Rc::new(Cell::new(vec![0.0; num_cores]));
        let is_vertical = init.settings.is_vertical.get();

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_visible(true);
        drawing_area.set_can_focus(false);
        setup_draw_func(&drawing_area, &core_values, &init.config);
        update_size(&drawing_area, num_cores, &init.config, is_vertical);

        watchers::spawn_watchers(&sender, cpuchart_config, &init.sysinfo);

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: String::new(),
                label: String::new(),
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: ConfigProperty::new(
                        wayle_config::schemas::styling::ColorValue::Token(CssToken::FgDefault),
                    ),
                    label_color: ConfigProperty::new(
                        wayle_config::schemas::styling::ColorValue::Token(CssToken::FgDefault),
                    ),
                    icon_background: ConfigProperty::new(
                        wayle_config::schemas::styling::ColorValue::Token(CssToken::BgBase),
                    ),
                    button_background: cpuchart_config.button_bg_color.clone(),
                    border_color: cpuchart_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: ConfigProperty::new(0),
                    show_icon: ConfigProperty::new(false),
                    show_label: ConfigProperty::new(true), // Must be true so label_container is visible
                    show_border: cpuchart_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => CpuChartMsg::LeftClick,
                BarButtonOutput::RightClick => CpuChartMsg::RightClick,
                BarButtonOutput::MiddleClick => CpuChartMsg::MiddleClick,
                BarButtonOutput::ScrollUp => CpuChartMsg::ScrollUp,
                BarButtonOutput::ScrollDown => CpuChartMsg::ScrollDown,
            });

        let model = Self {
            bar_button,
            drawing_area: drawing_area.clone(),
            core_values,
            config: init.config.clone(),
            is_vertical,
            dropdowns: init.dropdowns,
        };
        let bar_button_widget = model.bar_button.widget();

        if let Some(button_box) = bar_button_widget.child().and_downcast::<gtk4::Box>() {
            let mut child = button_box.first_child();
            while let Some(current) = child {
                if current.css_classes().iter().any(|c| c == "label-container") {
                    if let Some(label_container) = current.downcast_ref::<gtk4::Box>() {
                        label_container.append(&drawing_area);
                        break;
                    }
                }
                child = current.next_sibling();
            }
        }

        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let cpuchart_config = &self.config.config().modules.cpuchart;

        let action = match msg {
            CpuChartMsg::LeftClick => cpuchart_config.left_click.get(),
            CpuChartMsg::RightClick => cpuchart_config.right_click.get(),
            CpuChartMsg::MiddleClick => cpuchart_config.middle_click.get(),
            CpuChartMsg::ScrollUp => cpuchart_config.scroll_up.get(),
            CpuChartMsg::ScrollDown => cpuchart_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: CpuChartCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CpuChartCmd::Update(core_values) => {
                self.core_values.set(core_values);
                self.drawing_area.queue_draw();
            }
            CpuChartCmd::Resize => {
                let current_values = self.core_values.take();
                let num_cores = current_values.len();
                self.core_values.set(current_values);

                if num_cores > 0 {
                    update_size(
                        &self.drawing_area,
                        num_cores,
                        &self.config,
                        self.is_vertical,
                    );
                }
                self.drawing_area.queue_draw();
            }
        }
    }
}

fn setup_draw_func(
    drawing_area: &gtk4::DrawingArea,
    core_values: &Rc<Cell<Vec<f64>>>,
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
        let horizontal_padding = super::shared::rem_to_px(padding_rem, bar_scale);
        let direction = cpuchart_config.direction.get();
        let color = cpuchart_config.color.get();

        cr.translate(horizontal_padding, 0.0);

        let fill_color = super::shared::resolve_rgba(&color, &config_clone);

        let params = wayle_widgets::primitives::barchart::BarchartParams {
            bar_width,
            bar_spacing,
            fill_color,
        };

        wayle_widgets::primitives::barchart::draw_barchart(
            cr,
            &core_data,
            pixel_height,
            direction,
            &params,
        );

        values.set(core_data);
    });
}

fn update_size(
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
    let horizontal_padding = super::shared::rem_to_px(padding_rem, bar_scale);

    let total_width =
        (num_cores as f64 * (bar_width + bar_spacing)) - bar_spacing + (2.0 * horizontal_padding);

    if is_vertical {
        drawing_area.set_height_request(total_width as i32);
        drawing_area.set_width_request(20);
    } else {
        drawing_area.set_width_request(total_width as i32);
        drawing_area.set_height_request(20);
    }
}
