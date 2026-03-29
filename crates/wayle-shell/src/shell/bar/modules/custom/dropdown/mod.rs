mod item;

use gtk::{pango, prelude::*};
use relm4::{factory::FactoryVecDeque, gtk, prelude::*};

use self::item::{DropdownOptionInit, DropdownOptionItem};

/// Input messages for the dropdown picker.
#[derive(Debug)]
pub(crate) enum DropdownPickerInput {
    /// Populate the list. `active_item` is compared against each entry to
    /// determine which one gets the selected/checkmark treatment.
    Refresh {
        items: Vec<String>,
        active_item: String,
        ellipsize: pango::EllipsizeMode,
    },
    /// User activated a row at this index.
    ItemSelected(usize),
}

/// Output messages sent to the parent custom module.
#[derive(Debug)]
pub(crate) enum DropdownPickerOutput {
    /// The user selected an item (carries the item text).
    Selected(String),
}

pub(crate) struct CustomDropdownPicker {
    items_list: FactoryVecDeque<DropdownOptionItem>,
    /// Selectable item values, aligned with interactive rows only.
    /// Empty/placeholder rows are not included.
    cached_items: Vec<String>,
}

#[relm4::component(pub(crate))]
impl Component for CustomDropdownPicker {
    type Init = ();
    type Input = DropdownPickerInput;
    type Output = DropdownPickerOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            add_css_class: "custom-dropdown-picker",

            gtk::ScrolledWindow {
                set_vexpand: true,
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_max_content_height: 300,
                set_propagate_natural_height: true,

                #[local_ref]
                items_list -> gtk::ListBox {
                    add_css_class: "custom-dropdown-list",
                    set_activate_on_single_click: true,
                    set_selection_mode: gtk::SelectionMode::None,
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_box = gtk::ListBox::new();
        let picker_sender = sender.input_sender().clone();
        list_box.connect_row_activated(move |_, row| {
            if let Ok(index) = usize::try_from(row.index()) {
                picker_sender.emit(DropdownPickerInput::ItemSelected(index));
            }
        });

        let items_list = FactoryVecDeque::builder().launch(list_box).detach();

        let model = Self {
            items_list,
            cached_items: Vec::new(),
        };

        let items_list = model.items_list.widget();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            DropdownPickerInput::Refresh {
                items,
                active_item,
                ellipsize,
            } => {
                let mut guard = self.items_list.guard();
                guard.clear();
                if items.is_empty() {
                    guard.push_back(DropdownOptionInit {
                        label: String::from("(no items)"),
                        is_active: false,
                        interactive: false,
                        ellipsize,
                    });
                    self.cached_items.clear();
                } else {
                    for item in &items {
                        guard.push_back(DropdownOptionInit {
                            label: item.clone(),
                            is_active: item.trim() == active_item.trim(),
                            interactive: true,
                            ellipsize,
                        });
                    }
                    self.cached_items = items;
                }
            }
            DropdownPickerInput::ItemSelected(index) => {
                if let Some(item) = self.cached_items.get(index) {
                    sender
                        .output(DropdownPickerOutput::Selected(item.clone()))
                        .ok();
                }
            }
        }
    }
}
