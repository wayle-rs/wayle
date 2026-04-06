//! Bar dropdown settings: behavior toggles and appearance.

use std::sync::Arc;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigService;

use crate::{
    pages::helpers::{self, Keepalive, build_page_header, build_section},
    row::{SettingRow, SettingRowMsg},
};

#[allow(dead_code)]
pub struct BarDropdownPage {
    rows: Vec<Controller<SettingRow>>,
    keepalives: Vec<Keepalive>,
}

#[derive(Debug)]
pub enum BarDropdownPageMsg {
    ConfigChanged,
}

#[relm4::component(pub)]
impl SimpleComponent for BarDropdownPage {
    type Init = Arc<ConfigService>;
    type Input = BarDropdownPageMsg;
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

        content.append(&build_page_header("settings-page-bar-dropdown"));

        let mut rows = Vec::new();

        rows.extend(build_section(
            content,
            "settings-section-behavior",
            vec![
                helpers::toggle(&bar.dropdown_shadow, &mut keepalives),
                helpers::toggle(&bar.dropdown_autohide, &mut keepalives),
                helpers::toggle(&bar.dropdown_freeze_label, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-appearance",
            vec![helpers::percentage(&bar.dropdown_opacity, &mut keepalives)],
        ));

        let model = Self { rows, keepalives };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BarDropdownPageMsg::ConfigChanged => {
                for row in &self.rows {
                    row.emit(SettingRowMsg::Refresh);
                }
            }
        }
    }
}
