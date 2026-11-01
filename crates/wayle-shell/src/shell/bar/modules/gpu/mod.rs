mod factory;
mod helpers;
mod messages;
mod watchers;

use std::{rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{GpuCmd, GpuInit, GpuMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct GpuModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for GpuModule {
    type Init = GpuInit;
    type Input = GpuMsg;
    type Output = ();
    type CommandOutput = GpuCmd;

    view! {
        gtk::Box {
            add_css_class: "gpu",

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
        let gpu_config = &config.modules.gpu;

        let initial_label =
            helpers::format_label(&gpu_config.format.get(), &init.sysinfo.gpu.get());

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: gpu_config.icon_name.get().clone(),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: gpu_config.icon_color.clone(),
                    label_color: gpu_config.label_color.clone(),
                    icon_background: gpu_config.icon_bg_color.clone(),
                    button_background: gpu_config.button_bg_color.clone(),
                    border_color: gpu_config.border_color.clone(),
                    auto_icon_color: CssToken::Blue,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: gpu_config.label_max_length.clone(),
                    show_icon: gpu_config.icon_show.clone(),
                    show_label: gpu_config.label_show.clone(),
                    show_border: gpu_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => GpuMsg::LeftClick,
                BarButtonOutput::RightClick => GpuMsg::RightClick,
                BarButtonOutput::MiddleClick => GpuMsg::MiddleClick,
                BarButtonOutput::ScrollUp => GpuMsg::ScrollUp,
                BarButtonOutput::ScrollDown => GpuMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, gpu_config, &init.sysinfo);

        let model = Self {
            bar_button,
            config: init.config,
            dropdowns: init.dropdowns,
        };
        let bar_button = model.bar_button.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        let gpu_config = &self.config.config().modules.gpu;

        let action = match msg {
            GpuMsg::LeftClick => gpu_config.left_click.get(),
            GpuMsg::RightClick => gpu_config.right_click.get(),
            GpuMsg::MiddleClick => gpu_config.middle_click.get(),
            GpuMsg::ScrollUp => gpu_config.scroll_up.get(),
            GpuMsg::ScrollDown => gpu_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: GpuCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            GpuCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            GpuCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            GpuCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
