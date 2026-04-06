//! General settings page: fonts and tearing mode.

use std::sync::Arc;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigService;

use crate::{
    pages::helpers::{self, Keepalive, build_page_header, build_section},
    row::{SettingRow, SettingRowMsg},
};

#[allow(dead_code)]
pub struct GeneralPage {
    rows: Vec<Controller<SettingRow>>,
    keepalives: Vec<Keepalive>,
}

#[derive(Debug)]
pub enum GeneralPageMsg {
    ConfigChanged,
}

#[relm4::component(pub)]
impl SimpleComponent for GeneralPage {
    type Init = Arc<ConfigService>;
    type Input = GeneralPageMsg;
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
        config_service: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let general = &config_service.config().general;
        let content = &widgets.content;
        let mut keepalives: Vec<Keepalive> = Vec::new();

        content.append(&build_page_header("settings-page-general"));

        let mut rows = Vec::new();

        rows.extend(build_section(
            content,
            "settings-section-layout",
            vec![
                helpers::font(&general.font_sans, &mut keepalives),
                helpers::font(&general.font_mono, &mut keepalives),
                helpers::toggle(&general.tearing_mode, &mut keepalives),
            ],
        ));

        let model = Self { rows, keepalives };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            GeneralPageMsg::ConfigChanged => {
                for row in &self.rows {
                    row.emit(SettingRowMsg::Refresh);
                }
            }
        }
    }
}
