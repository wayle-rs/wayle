mod factory;
mod helpers;
mod messages;
mod watchers;

use std::{path::Path, rc::Rc, sync::Arc};

use gtk::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, ConfigService, schemas::styling::CssToken};
use wayle_widgets::prelude::{
    BarButton, BarButtonBehavior, BarButtonColors, BarButtonInit, BarButtonInput, BarButtonOutput,
};

pub(crate) use self::{
    factory::Factory,
    messages::{StorageCmd, StorageInit, StorageMsg},
};
use crate::shell::bar::dropdowns::{self, DropdownRegistry};

pub(crate) struct StorageModule {
    bar_button: Controller<BarButton>,
    config: Arc<ConfigService>,
    dropdowns: Rc<DropdownRegistry>,
}

#[relm4::component(pub(crate))]
impl Component for StorageModule {
    type Init = StorageInit;
    type Input = StorageMsg;
    type Output = ();
    type CommandOutput = StorageCmd;

    view! {
        gtk::Box {
            add_css_class: "storage",

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
        let storage_config = &config.modules.storage;

        let disks = init.sysinfo.disks.get();
        let target = storage_config.mount_point.get();
        let target_path = Path::new(&target);

        let initial_label = disks
            .iter()
            .find(|d| d.mount_point == target_path)
            .map(|d| helpers::format_label(&storage_config.format.get(), d))
            .unwrap_or_else(|| String::from("--"));

        let bar_button = BarButton::builder()
            .launch(BarButtonInit {
                icon: storage_config.icon_name.get().clone(),
                label: initial_label,
                tooltip: None,
                colors: BarButtonColors {
                    icon_color: storage_config.icon_color.clone(),
                    label_color: storage_config.label_color.clone(),
                    icon_background: storage_config.icon_bg_color.clone(),
                    button_background: storage_config.button_bg_color.clone(),
                    border_color: storage_config.border_color.clone(),
                    auto_icon_color: CssToken::Yellow,
                },
                behavior: BarButtonBehavior {
                    label_max_chars: storage_config.label_max_length.clone(),
                    show_icon: storage_config.icon_show.clone(),
                    show_label: storage_config.label_show.clone(),
                    show_border: storage_config.border_show.clone(),
                    visible: ConfigProperty::new(true),
                },
                settings: init.settings,
            })
            .forward(sender.input_sender(), |output| match output {
                BarButtonOutput::LeftClick => StorageMsg::LeftClick,
                BarButtonOutput::RightClick => StorageMsg::RightClick,
                BarButtonOutput::MiddleClick => StorageMsg::MiddleClick,
                BarButtonOutput::ScrollUp => StorageMsg::ScrollUp,
                BarButtonOutput::ScrollDown => StorageMsg::ScrollDown,
            });

        watchers::spawn_watchers(&sender, storage_config, &init.sysinfo);

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
        let storage_config = &self.config.config().modules.storage;

        let action = match msg {
            StorageMsg::LeftClick => storage_config.left_click.get(),
            StorageMsg::RightClick => storage_config.right_click.get(),
            StorageMsg::MiddleClick => storage_config.middle_click.get(),
            StorageMsg::ScrollUp => storage_config.scroll_up.get(),
            StorageMsg::ScrollDown => storage_config.scroll_down.get(),
        };

        dropdowns::dispatch_click(&action, &self.dropdowns, &self.bar_button);
    }

    fn update_cmd(&mut self, msg: StorageCmd, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            StorageCmd::UpdateLabel(label) => {
                self.bar_button.emit(BarButtonInput::SetLabel(label));
            }
            StorageCmd::UpdateIcon(icon) => {
                self.bar_button.emit(BarButtonInput::SetIcon(icon));
            }
            StorageCmd::UpdateThresholdColors(colors) => {
                self.bar_button
                    .emit(BarButtonInput::SetThresholdColors(colors));
            }
        }
    }
}
