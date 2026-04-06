//! Bar general settings: layout, appearance, spacing, and border.

use std::sync::Arc;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigService;

use crate::{
    pages::helpers::{self, Keepalive, build_page_header, build_section},
    row::{SettingRow, SettingRowMsg},
};

#[allow(dead_code)]
pub struct BarGeneralPage {
    rows: Vec<Controller<SettingRow>>,
    keepalives: Vec<Keepalive>,
}

#[derive(Debug)]
pub enum BarGeneralPageMsg {
    ConfigChanged,
}

#[relm4::component(pub)]
impl SimpleComponent for BarGeneralPage {
    type Init = Arc<ConfigService>;
    type Input = BarGeneralPageMsg;
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
        let bar = &config_service.config().bar;
        let content = &widgets.content;
        let mut keepalives: Vec<Keepalive> = Vec::new();

        content.append(&build_page_header("settings-page-bar-general"));

        let mut rows = Vec::new();

        rows.extend(build_section(
            content,
            "settings-section-layout",
            vec![
                helpers::enum_select(&bar.location, &mut keepalives),
                helpers::scale(&bar.scale, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-appearance",
            vec![
                helpers::percentage(&bar.background_opacity, &mut keepalives),
                helpers::enum_select(&bar.rounding, &mut keepalives),
                helpers::enum_select(&bar.shadow, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-spacing",
            vec![
                helpers::spacing(&bar.inset_edge, &mut keepalives),
                helpers::spacing(&bar.inset_ends, &mut keepalives),
                helpers::spacing(&bar.padding, &mut keepalives),
                helpers::spacing(&bar.padding_ends, &mut keepalives),
                helpers::spacing(&bar.module_gap, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-border",
            vec![
                helpers::number_u8(&bar.border_width, &mut keepalives),
                helpers::enum_select(&bar.border_location, &mut keepalives),
            ],
        ));

        let model = Self { rows, keepalives };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BarGeneralPageMsg::ConfigChanged => {
                for row in &self.rows {
                    row.emit(SettingRowMsg::Refresh);
                }
            }
        }
    }
}
