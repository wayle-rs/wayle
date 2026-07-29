use gtk::prelude::*;
use relm4::{gtk, prelude::*};
use wayle_mullvad::{NetworkCity, NetworkTarget};

use super::{
    city_item::{CityItem, CityItemInit, CityItemInput, CityItemOutput},
    helpers,
};
use crate::i18n::t;

/// A country node: expands to a list of cities, or selects the country
/// directly (the daemon then picks a relay within it).
pub(super) struct CountryItem {
    name: String,
    flag: String,
    target: NetworkTarget,
    expanded: bool,
    hovered: bool,
    index: DynamicIndex,
    cities: FactoryVecDeque<CityItem>,
}

pub(super) struct CountryItemInit {
    pub name: String,
    pub code: String,
    pub cities: Vec<NetworkCity>,
}

#[derive(Debug)]
pub(super) enum CountryItemInput {
    ToggleExpanded,
    Hovered(bool),
    Collapse,
    CitySelect(NetworkTarget),
    CityExpanded(DynamicIndex),
}

#[derive(Debug)]
pub(crate) enum CountryItemOutput {
    Select(NetworkTarget),
    Expanded(DynamicIndex),
}

#[relm4::factory(pub(super))]
impl FactoryComponent for CountryItem {
    type Init = CountryItemInit;
    type Input = CountryItemInput;
    type Output = CountryItemOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "mullvad-country",
            set_orientation: gtk::Orientation::Vertical,

            #[name = "header"]
            gtk::Overlay {
                #[wrap(Some)]
                set_child = &gtk::Button {
                    add_css_class: "mullvad-expand",
                    set_hexpand: true,
                    set_cursor_from_name: Some("pointer"),
                    connect_clicked => CountryItemInput::ToggleExpanded,

                    gtk::Box {
                        gtk::Image {
                            add_css_class: "mullvad-country-flag",
                            set_icon_name: Some(self.flag.as_str()),
                            set_valign: gtk::Align::Center,
                        },
                        gtk::Label {
                            add_css_class: "mullvad-country-name",
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                            set_label: &self.name,
                        },
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
            cities_widget -> gtk::Box {
                add_css_class: "mullvad-cities",
                set_orientation: gtk::Orientation::Vertical,
                #[watch]
                set_visible: self.expanded,
            },
        }
    }

    fn init_model(init: Self::Init, index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let country_code = init.code.clone();

        let mut cities = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), |out| match out {
                CityItemOutput::Select(target) => CountryItemInput::CitySelect(target),
                CityItemOutput::Expanded(index) => CountryItemInput::CityExpanded(index),
            });

        {
            let mut guard = cities.guard();
            for city in init.cities {
                guard.push_back(CityItemInit {
                    name: city.name,
                    target: NetworkTarget::city(country_code.clone(), city.code),
                    relays: city.networks,
                });
            }
        }

        Self {
            name: init.name,
            flag: helpers::flag_icon(&init.code),
            target: NetworkTarget::country(init.code),
            expanded: false,
            hovered: false,
            index: index.clone(),
            cities,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let cities_widget = self.cities.widget();
        let widgets = view_output!();

        let target = self.target.clone();
        let out = sender.output_sender().clone();
        widgets.select_btn.connect_clicked(move |_| {
            out.emit(CountryItemOutput::Select(target.clone()));
        });

        let hover = gtk::EventControllerMotion::new();
        let enter = sender.input_sender().clone();
        hover.connect_enter(move |_, _, _| {
            enter.emit(CountryItemInput::Hovered(true));
        });
        let leave = sender.input_sender().clone();
        hover.connect_leave(move |_| {
            leave.emit(CountryItemInput::Hovered(false));
        });
        widgets.header.add_controller(hover);

        // Reset hover on unmap: GTK may not deliver a leave when the popover is
        // popped down with the pointer over the row.
        let unmap = sender.input_sender().clone();
        widgets.header.connect_unmap(move |_| {
            unmap.emit(CountryItemInput::Hovered(false));
        });

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            CountryItemInput::ToggleExpanded => {
                self.expanded = !self.expanded;
                if self.expanded {
                    let _ = sender.output(CountryItemOutput::Expanded(self.index.clone()));
                } else {
                    self.collapse_cities();
                }
            }
            CountryItemInput::Hovered(hovered) => {
                self.hovered = hovered;
            }
            CountryItemInput::Collapse => {
                self.expanded = false;
                self.collapse_cities();
            }
            CountryItemInput::CityExpanded(expanded) => {
                let expanded = expanded.current_index();
                for i in 0..self.cities.len() {
                    if i != expanded {
                        self.cities.send(i, CityItemInput::Collapse);
                    }
                }
            }
            CountryItemInput::CitySelect(target) => {
                let _ = sender.output(CountryItemOutput::Select(target));
            }
        }
    }
}

impl CountryItem {
    fn collapse_cities(&self) {
        for i in 0..self.cities.len() {
            self.cities.send(i, CityItemInput::Collapse);
        }
    }
}
