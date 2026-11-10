use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_mullvad::{MullvadNetwork, NetworkTarget};

use super::relay_item::{RelayItem, RelayItemInit, RelayItemOutput};
use crate::i18n::t;

/// A city node: expands to a list of relays, or selects the city directly.
pub(super) struct CityItem {
    name: String,
    target: NetworkTarget,
    expanded: bool,
    hovered: bool,
    index: DynamicIndex,
    relays: FactoryVecDeque<RelayItem>,
}

pub(super) struct CityItemInit {
    pub name: String,
    pub target: NetworkTarget,
    pub relays: Vec<MullvadNetwork>,
}

#[derive(Debug)]
pub(super) enum CityItemInput {
    ToggleExpanded,
    Hovered(bool),
    Collapse,
    RelaySelected(NetworkTarget),
}

#[derive(Debug)]
pub(super) enum CityItemOutput {
    Select(NetworkTarget),
    Expanded(DynamicIndex),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for CityItem {
    type Init = CityItemInit;
    type Input = CityItemInput;
    type Output = CityItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "mullvad-city",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Overlay {
                #[wrap(Some)]
                set_child = &gtk::Button {
                    add_css_class: "mullvad-expand",
                    set_hexpand: true,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => CityItemInput::ToggleExpanded,

                    gtk::Label {
                        add_css_class: "mullvad-city-name",
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_label: &self.name,
                    },
                },

                #[name = "select_btn"]
                add_overlay = &gtk::Button {
                    add_css_class: "mullvad-select-btn",
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,
                    set_cursor_from_name: Some("pointer"),
                    #[watch]
                    set_visible: self.hovered,
                    set_label: &t!("dropdown-mullvad-select"),
                },
            },

            #[local_ref]
            relays_widget -> gtk::Box {
                add_css_class: "mullvad-relays",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: self.expanded,
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut relays = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |out| match out {
                RelayItemOutput::Selected(target) => CityItemInput::RelaySelected(target),
            });

        {
            let mut guard = relays.guard();
            for relay in init.relays {
                guard.push_back(RelayItemInit {
                    hostname: relay.hostname.clone(),
                    active: relay.active,
                    target: NetworkTarget::relay(
                        relay.country_code,
                        relay.city_code,
                        relay.hostname,
                    ),
                });
            }
        }

        Self {
            name: init.name,
            target: init.target,
            expanded: false,
            hovered: false,
            index: index.clone(),
            relays,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let relays_widget = self.relays.widget();
        let widgets = view_output!();

        let target = self.target.clone();
        let out = sender.output_sender().clone();
        widgets.select_btn.connect_clicked(move |_| {
            out.emit(CityItemOutput::Select(target.clone()));
        });

        let hover = gtk::EventControllerMotion::new();
        let enter = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            enter.emit(CityItemInput::Hovered(true));
        });
        let leave = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave.emit(CityItemInput::Hovered(false));
        });
        widgets.header.add_controller(hover);

        // Reset hover on unmap: GTK may not deliver a leave when the popover is
        // popped down with the pointer over the row.
        let unmap = sender.input_sender().clone();
        widgets.header.connect_unmap(move |_| {
            unmap.emit(CityItemInput::Hovered(false));
        });

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            CityItemInput::ToggleExpanded => {
                self.expanded = !self.expanded;
                if self.expanded {
                    let _ = sender.output(CityItemOutput::Expanded(self.index.clone()));
                }
            }
            CityItemInput::Hovered(hovered) => {
                self.hovered = hovered;
            }
            CityItemInput::Collapse => {
                self.expanded = false;
            }
            CityItemInput::RelaySelected(target) => {
                let _ = sender.output(CityItemOutput::Select(target));
            }
        }
    }
}
