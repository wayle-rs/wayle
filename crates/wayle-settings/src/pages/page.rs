//! Generic settings page component. Each page is just a PageSpec
//! describing its header and sections.

use gtk4::prelude::*;
use relm4::prelude::*;

use super::helpers::{self, Keepalive, PageSpec};
use crate::row::{SettingRow, SettingRowMsg};

#[allow(dead_code)]
pub(crate) struct SettingsPage {
    rows: Vec<Controller<SettingRow>>,
    keepalives: Vec<Keepalive>,
}

#[derive(Debug)]
pub(crate) enum SettingsPageMsg {
    Refresh,
}

#[relm4::component(pub(crate))]
impl SimpleComponent for SettingsPage {
    type Init = PageSpec;
    type Input = SettingsPageMsg;
    type Output = ();

    view! {
        gtk4::ScrolledWindow {
            set_hexpand: true,
            set_vexpand: true,

            #[name = "content"]
            gtk4::Box {
                set_orientation: gtk4::Orientation::Vertical,
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

        content.append(&helpers::build_page_header(spec.header_key));

        let (rows, keepalives) = helpers::build_sections(content, spec.sections);

        let model = Self { rows, keepalives };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            SettingsPageMsg::Refresh => {
                for row in &self.rows {
                    row.emit(SettingRowMsg::Refresh);
                }
            }
        }
    }
}
