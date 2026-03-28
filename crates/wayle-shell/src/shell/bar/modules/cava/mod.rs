mod factory;
mod helpers;
mod messages;
mod methods;
mod rendering;
mod watchers;

use std::{cell::Cell, rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use tracing::{error, info};
use wayle_cava::CavaService;
use wayle_config::{ConfigProperty, ConfigService};
use wayle_widgets::{
    WatcherToken,
    prelude::{BarContainer, BarContainerBehavior, BarContainerColors, BarContainerInit},
    primitives::barchart::calculate_widget_length,
};

use self::messages::CavaMsg;
pub(crate) use self::{
    factory::Factory,
    messages::{CavaCmd, CavaInit},
};
use crate::shell::bar::dropdowns::{DropdownRegistry, dispatch_click_widget};

/// Audio frequency visualizer rendered via cairo on a `DrawingArea`.
pub(crate) struct CavaModule {
    container: Controller<BarContainer>,
    drawing_area: gtk::DrawingArea,
    frame_data: Rc<Cell<Vec<f64>>>,
    frame_watcher: WatcherToken,
    is_vertical: bool,
    cava: Option<Arc<CavaService>>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
    container_widget: gtk::Box,
}

#[relm4::component(pub(crate))]
impl Component for CavaModule {
    type Init = CavaInit;
    type Input = CavaMsg;
    type Output = ();
    type CommandOutput = CavaCmd;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "cava",
            set_cursor_from_name: Some("pointer"),

            #[local_ref]
            container -> gtk::Box {
                #[local_ref]
                drawing_area -> gtk::DrawingArea {
                    set_cursor_from_name: Some("pointer"),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let is_vertical = init.settings.is_vertical.get();
        let config = &init.config;
        let full_config = config.config();
        let cava_config = &full_config.modules.cava;
        let styling_config = &full_config.styling;
        let bar_config = &full_config.bar;

        let container = BarContainer::builder()
            .launch(BarContainerInit {
                colors: BarContainerColors {
                    background: cava_config.button_bg_color.clone(),
                    border_color: cava_config.border_color.clone(),
                },
                behavior: BarContainerBehavior {
                    show_border: cava_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                is_vertical: init.settings.is_vertical.clone(),
                theme_provider: styling_config.theme_provider.clone(),
                border_width: bar_config.button_border_width.clone(),
                border_location: bar_config.button_border_location.clone(),
            })
            .detach();

        let bars = cava_config.bars.get().value();
        let bar_width = cava_config.bar_width.get();
        let bar_gap = cava_config.bar_gap.get();
        let bar_scale = bar_config.scale.get().value();
        let internal_padding = cava_config.internal_padding.get().value();
        let padding_px = helpers::rem_to_px(internal_padding, bar_scale);

        let drawing_area = gtk::DrawingArea::new();
        let length = calculate_widget_length(bars, bar_width, bar_gap, padding_px);

        if is_vertical {
            drawing_area.set_size_request(-1, length);
            drawing_area.set_hexpand(true);
        } else {
            drawing_area.set_size_request(length, -1);
            drawing_area.set_vexpand(true);
        }

        let frame_data: Rc<Cell<Vec<f64>>> = Rc::new(Cell::new(vec![0.0; bars as usize]));

        Self::setup_draw_func(&drawing_area, &frame_data, is_vertical, config);

        let config_clone = init.config.clone();
        sender.oneshot_command(async move {
            match helpers::build_cava_service(&config_clone).await {
                Ok(service) => CavaCmd::ServiceReady(service),
                Err(err) => {
                    error!(error = %err, "cava service failed to start");
                    CavaCmd::ServiceFailed
                }
            }
        });

        watchers::spawn_config_watchers(
            &sender,
            init.settings.is_vertical,
            &init.config,
            &init.wallpaper,
        );

        let scroll_sensitivity = init.settings.scroll_sensitivity;
        Self::attach_click_gesture(&root, &sender);
        Self::attach_scroll_controller(&root, &sender, scroll_sensitivity);

        let model = Self {
            container,
            drawing_area: drawing_area.clone(),
            frame_data,
            frame_watcher: WatcherToken::new(),
            is_vertical,
            cava: None,
            config: init.config.clone(),
            dropdowns: init.dropdowns,
            container_widget: root.clone(),
        };
        let container = model.container.widget();
        let drawing_area = &model.drawing_area;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: CavaMsg, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let cava_config = &self.config.config().modules.cava;
        let action = match msg {
            CavaMsg::LeftClick => cava_config.left_click.get(),
            CavaMsg::RightClick => cava_config.right_click.get(),
            CavaMsg::MiddleClick => cava_config.middle_click.get(),
            CavaMsg::ScrollUp => cava_config.scroll_up.get(),
            CavaMsg::ScrollDown => cava_config.scroll_down.get(),
        };
        dispatch_click_widget(&action, &self.dropdowns, &self.container_widget);
    }

    fn update_cmd(&mut self, msg: CavaCmd, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            CavaCmd::ServiceReady(service) => {
                info!("cava service started");
                let token = self.frame_watcher.reset();
                watchers::spawn_frame_watcher(&sender, &service, token);
                self.cava = Some(service);
            }
            CavaCmd::ServiceFailed => {}
            CavaCmd::ServiceConfigChanged => {
                self.cava = None;
                let bars = self.config.config().modules.cava.bars.get().value();
                self.frame_data.set(vec![0.0; bars as usize]);
                self.update_size();
                Self::setup_draw_func(
                    &self.drawing_area,
                    &self.frame_data,
                    self.is_vertical,
                    &self.config,
                );

                let config_clone = self.config.clone();
                sender.oneshot_command(async move {
                    match helpers::build_cava_service(&config_clone).await {
                        Ok(service) => CavaCmd::ServiceReady(service),
                        Err(err) => {
                            error!(error = %err, "cava service restart failed");
                            CavaCmd::ServiceFailed
                        }
                    }
                });
            }
            CavaCmd::Frame(values) => {
                self.frame_data.set(values);
                self.drawing_area.queue_draw();
            }
            CavaCmd::StylingChanged => {
                self.update_size();
                Self::setup_draw_func(
                    &self.drawing_area,
                    &self.frame_data,
                    self.is_vertical,
                    &self.config,
                );
                self.drawing_area.queue_draw();
            }
            CavaCmd::OrientationChanged(is_vertical) => {
                self.is_vertical = is_vertical;
                self.update_size();
                Self::setup_draw_func(
                    &self.drawing_area,
                    &self.frame_data,
                    self.is_vertical,
                    &self.config,
                );
                self.drawing_area.queue_draw();
            }
        }
    }
}
