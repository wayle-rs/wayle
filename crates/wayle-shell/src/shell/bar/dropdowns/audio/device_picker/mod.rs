mod device_item;
mod messages;
mod methods;
mod watchers;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};
use wayle_audio::AudioService;
use wayle_config::ConfigService;

use self::device_item::DeviceOptionItem;
pub(super) use self::messages::*;
use crate::shell::bar::dropdowns::audio::VolumeSectionKind;

pub(crate) struct DevicePicker {
    audio: Arc<AudioService>,
    config: Arc<ConfigService>,
    kind: VolumeSectionKind,
    title: String,
    devices: FactoryVecDeque<DeviceOptionItem>,
    cached_output_devices: Vec<Arc<wayle_audio::core::device::output::OutputDevice>>,
    cached_input_devices: Vec<Arc<wayle_audio::core::device::input::InputDevice>>,
}

#[relm4::component(pub(crate))]
impl Component for DevicePicker {
    type Init = DevicePickerInit;
    type Input = DevicePickerInput;
    type Output = DevicePickerOutput;
    type CommandOutput = DevicePickerCmd;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                add_css_class: "picker-header",

                #[template]
                wayle_widgets::prelude::GhostIconButton {
                    add_css_class: "picker-back",
                    set_icon_name: "ld-arrow-left-symbolic",
                    connect_clicked => DevicePickerInput::BackClicked,
                },

                gtk::Label {
                    add_css_class: "picker-title",
                    #[watch]
                    set_label: &model.title,
                },
            },

            gtk::ScrolledWindow {
                add_css_class: "picker-body",
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,

                #[local_ref]
                device_list -> gtk::ListBox {
                    add_css_class: "audio-device-list",
                    set_activate_on_single_click: true,
                    set_selection_mode: gtk::SelectionMode::None,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let device_list = gtk::ListBox::new();
        let picker_sender = sender.input_sender().clone();
        device_list.connect_row_activated(move |_list_box, row| {
            if let Ok(index) = usize::try_from(row.index()) {
                picker_sender.emit(DevicePickerInput::DeviceSelected(index));
            }
        });

        let devices = FactoryVecDeque::builder().launch(device_list).detach();

        watchers::spawn(&sender, &init.audio, &init.config, init.kind);

        let initial_list = Self::build_device_list(&init.audio, &init.config, init.kind);
        let (cached_output, cached_input) = match init.kind {
            VolumeSectionKind::Output => (
                methods::visible_output_devices(
                    &init.audio.output_devices.get(),
                    methods::hide_unavailable_outputs(&init.config),
                ),
                Vec::new(),
            ),
            VolumeSectionKind::Input => (Vec::new(), init.audio.input_devices.get()),
        };

        let mut model = Self {
            audio: init.audio,
            config: init.config,
            kind: init.kind,
            title: init.title,
            devices,
            cached_output_devices: cached_output,
            cached_input_devices: cached_input,
        };

        model.apply_device_list(initial_list);

        let device_list = model.devices.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            DevicePickerInput::DeviceSelected(index) => {
                self.select_device(index, &sender);
            }
            DevicePickerInput::BackClicked => {
                let _ = sender.output(DevicePickerOutput::NavigateBack);
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            DevicePickerCmd::DevicesChanged(list) => {
                match self.kind {
                    VolumeSectionKind::Output => {
                        self.cached_output_devices = methods::visible_output_devices(
                            &self.audio.output_devices.get(),
                            methods::hide_unavailable_outputs(&self.config),
                        );
                    }
                    VolumeSectionKind::Input => {
                        self.cached_input_devices = self.audio.input_devices.get();
                    }
                }
                self.apply_device_list(list);
            }
        }
    }
}
