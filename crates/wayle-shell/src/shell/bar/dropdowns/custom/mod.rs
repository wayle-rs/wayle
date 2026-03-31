mod factory;
mod picker;

use std::sync::Arc;

use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_config::{ConfigService, schemas::dropdowns::CustomDropdownDefinition};
use wayle_widgets::prelude::*;

pub(super) use self::factory::create;
use self::picker::{PickerSection, PickerSectionInit};
use super::scaled_dimension;

pub(crate) struct CustomDropdown {
    scaled_width: i32,
    title: String,
    icon: Option<String>,
    _picker: Option<Controller<PickerSection>>,
}

pub(crate) struct CustomDropdownInit {
    pub definition: CustomDropdownDefinition,
    pub config: Arc<ConfigService>,
}

#[relm4::component(pub(crate))]
impl Component for CustomDropdown {
    type Init = CustomDropdownInit;
    type Input = ();
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Popover {
            set_css_classes: &["dropdown", "custom-dropdown"],
            set_has_arrow: false,
            #[watch]
            set_width_request: model.scaled_width,

            #[template]
            Dropdown {
                #[template]
                DropdownHeader {
                    #[template_child]
                    icon {
                        #[watch]
                        set_visible: model.icon.is_some(),
                        #[watch]
                        set_icon_name: model.icon.as_deref(),
                    },
                    #[template_child]
                    label {
                        #[watch]
                        set_label: &model.title,
                    },
                },

                #[template]
                DropdownContent {
                    #[local_ref]
                    picker_widget -> gtk::Box {},
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let definition = init.definition;
        let scale = init.config.config().bar.scale.get().value();
        let scaled_width = scaled_dimension(definition.width, scale);

        // Build picker section from first picker section definition.
        let picker_config = definition.sections.iter().find_map(|s| s.as_picker());

        let picker = picker_config.map(|config| {
            PickerSection::builder()
                .launch(PickerSectionInit { config })
                .detach()
        });

        let picker_widget = picker
            .as_ref()
            .map(|p| p.widget().clone())
            .unwrap_or_else(|| gtk::Box::new(gtk::Orientation::Vertical, 0));

        let model = Self {
            scaled_width,
            title: definition.title.unwrap_or_default(),
            icon: definition.icon,
            _picker: picker,
        };

        let widgets = view_output!();

        // Reload picker content each time the popover becomes visible.
        if let Some(picker) = &model._picker {
            let picker_sender = picker.sender().clone();
            root.connect_notify(Some("visible"), move |popover, _| {
                if popover.is_visible() {
                    picker_sender.send(picker::PickerMsg::Reload).ok();
                }
            });
        }

        ComponentParts { model, widgets }
    }
}
