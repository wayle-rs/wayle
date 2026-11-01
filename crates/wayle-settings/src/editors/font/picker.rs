//! Searchable popover of installed font families.
//!
//! Backed by a virtualized `ListView` so incremental search stays responsive
//! across the full system font list.

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;
use wayle_i18n::t;

use super::helpers::{build_factory, build_family_list, build_name_expression};

pub(super) struct FontPicker {
    pub(super) property: ConfigProperty<String>,
    pub(super) filter: gtk::StringFilter,
    pub(super) filter_model: gtk::FilterListModel,
}

#[derive(Debug)]
pub(super) enum FontPickerMsg {
    SearchChanged(String),
    Activated(u32),
    Shown,
}

#[relm4::component(pub(super))]
impl Component for FontPicker {
    type Init = ConfigProperty<String>;
    type Input = FontPickerMsg;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Popover {
            add_css_class: "font-picker-popover",

            connect_show => FontPickerMsg::Shown,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name = "search_entry"]
                gtk::SearchEntry {
                    set_placeholder_text: Some(&t("settings-font-search")),
                    set_valign: gtk::Align::Center,

                    connect_search_changed[sender] => move |entry| {
                        sender.input(FontPickerMsg::SearchChanged(entry.text().into()));
                    },
                },

                gtk::ScrolledWindow {
                    add_css_class: "font-picker-scroll",
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[name = "list_view"]
                    gtk::ListView {
                        add_css_class: "font-picker-list",
                        set_single_click_activate: true,
                        set_show_separators: false,

                        connect_activate[sender] => move |_list_view, position| {
                            sender.input(FontPickerMsg::Activated(position));
                        },
                    },
                },
            },
        }
    }

    fn init(
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let string_list = build_family_list();

        let filter = gtk::StringFilter::builder()
            .expression(build_name_expression())
            .match_mode(gtk::StringFilterMatchMode::Substring)
            .ignore_case(true)
            .build();

        let filter_model = gtk::FilterListModel::new(Some(string_list), Some(filter.clone()));
        let selection = gtk::NoSelection::new(Some(filter_model.clone()));
        let factory = build_factory();

        let widgets = view_output!();

        widgets.list_view.set_model(Some(&selection));
        widgets.list_view.set_factory(Some(&factory));

        if let Some(icon) = widgets.search_entry.first_child() {
            icon.set_halign(gtk::Align::Center);
            icon.set_valign(gtk::Align::Center);
            icon.add_css_class("font-picker-search-icon");
        }

        let model = Self {
            property,
            filter,
            filter_model,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            FontPickerMsg::SearchChanged(query) => self.apply_search(&query),

            FontPickerMsg::Activated(position) => self.select_and_close(position, root),

            FontPickerMsg::Shown => self.reset_search(),
        }
    }
}
