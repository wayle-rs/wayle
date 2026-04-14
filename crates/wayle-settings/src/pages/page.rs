//! Generic settings page component. Each page is just a PageSpec
//! describing its header and sections.

use relm4::{gtk, gtk::prelude::*, prelude::*};

use crate::{
    pages::{
        layout::{build_page_header, build_sections},
        spec::PageSpec,
    },
    row::SettingRow,
};

#[allow(dead_code)]
pub(crate) struct SettingsPage {
    rows: Vec<Controller<SettingRow>>,
}

#[relm4::component(pub(crate))]
impl SimpleComponent for SettingsPage {
    type Init = PageSpec;
    type Input = ();
    type Output = ();

    view! {
        gtk::ScrolledWindow {
            set_hexpand: true,
            set_vexpand: true,

            #[name = "content"]
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                add_css_class: "settings-page",
            },
        }
    }

    fn init(
        spec: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let content = &widgets.content;

        content.append(&build_page_header(spec.header_key));

        let rows = build_sections(content, spec.sections);

        let model = Self { rows };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
