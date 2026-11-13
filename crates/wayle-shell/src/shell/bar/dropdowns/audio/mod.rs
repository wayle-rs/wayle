mod device_picker;
mod factory;
pub(crate) mod helpers;
mod main_section;
mod messages;
mod watchers;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_widgets::prelude::*;

pub(super) use self::factory::Factory;
pub(crate) use self::main_section::default_devices::volume_section::VolumeSectionKind;
use self::{
    device_picker::{DevicePicker, DevicePickerInit, DevicePickerOutput},
    main_section::{MainSection, MainSectionInit, MainSectionOutput},
    messages::{AudioDropdownCmd, AudioDropdownInit, AudioDropdownMsg},
};
use crate::{i18n::t, shell::bar::dropdowns::scaled_dimension};

const BASE_WIDTH: f32 = 382.0;
const BASE_HEIGHT: f32 = 512.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AudioPage {
    Main,
    OutputDevices,
    InputDevices,
}

impl AudioPage {
    fn name(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::OutputDevices => "output",
            Self::InputDevices => "input",
        }
    }
}

pub(crate) struct AudioDropdown {
    scaled_width: i32,
    scaled_height: i32,
    active_page: AudioPage,
    main_section: Controller<MainSection>,
    output_picker: Controller<DevicePicker>,
    input_picker: Controller<DevicePicker>,
}

#[relm4::component(pub(crate))]
impl Component for AudioDropdown {
    type Init = AudioDropdownInit;
    type Input = AudioDropdownMsg;
    type Output = ();
    type CommandOutput = AudioDropdownCmd;

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "audio-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,
            #[watch]
            set_height_request: model.scaled_height,

            #[template]
            Dropdown {

                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        set_visible: true,
                        set_icon_name: Some("ld-volume-2-symbolic"),
                    },
                    #[template_child]
                    label {
                        set_label: &t!("dropdown-audio-title"),
                    },
                },

                #[name = "stack"]
                gtk::Stack {
                    add_css_class: "dropdown-content",
                    set_vexpand: true,
                    set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                    set_transition_duration: 200,
                    #[local_ref]
                    add_named[Some("main")] = main_section_widget -> gtk::Box {},

                    #[local_ref]
                    add_named[Some("output")] = output_picker_widget -> gtk::Box {},

                    #[local_ref]
                    add_named[Some("input")] = input_picker_widget -> gtk::Box {},

                    #[watch]
                    set_visible_child_name: model.active_page.name(),
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let main_section = MainSection::builder()
            .launch(MainSectionInit {
                audio: init.audio.clone(),
                config: init.config.clone(),
            })
            .forward(sender.input_sender(), AudioDropdownMsg::MainSection);

        let output_picker = DevicePicker::builder()
            .launch(DevicePickerInit {
                audio: init.audio.clone(),
                config: init.config.clone(),
                kind: VolumeSectionKind::Output,
                title: t!("dropdown-audio-output-devices"),
            })
            .forward(sender.input_sender(), AudioDropdownMsg::OutputPicker);

        let input_picker = DevicePicker::builder()
            .launch(DevicePickerInit {
                audio: init.audio.clone(),
                config: init.config.clone(),
                kind: VolumeSectionKind::Input,
                title: t!("dropdown-audio-input-devices"),
            })
            .forward(sender.input_sender(), AudioDropdownMsg::InputPicker);

        let scale = init.config.config().styling.scale.get().value();
        watchers::spawn(&sender, &init.config);

        let model = Self {
            scaled_width: scaled_dimension(BASE_WIDTH, scale),
            scaled_height: scaled_dimension(BASE_HEIGHT, scale),
            active_page: AudioPage::Main,
            main_section,
            output_picker,
            input_picker,
        };

        let main_section_widget = model.main_section.widget();
        let output_picker_widget = model.output_picker.widget();
        let input_picker_widget = model.input_picker.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            AudioDropdownMsg::MainSection(output) => match output {
                MainSectionOutput::ShowOutputDevices => {
                    self.active_page = AudioPage::OutputDevices;
                }
                MainSectionOutput::ShowInputDevices => {
                    self.active_page = AudioPage::InputDevices;
                }
            },
            AudioDropdownMsg::OutputPicker(DevicePickerOutput::NavigateBack)
            | AudioDropdownMsg::InputPicker(DevicePickerOutput::NavigateBack) => {
                self.active_page = AudioPage::Main;
            }
        }
    }

    fn update_cmd(
        &mut self,
        msg: AudioDropdownCmd,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AudioDropdownCmd::ScaleChanged(scale) => {
                self.scaled_width = scaled_dimension(BASE_WIDTH, scale);
                self.scaled_height = scaled_dimension(BASE_HEIGHT, scale);
            }
        }
    }
}
