//! Bar button settings: style, icons, labels, borders, and button groups.

use std::sync::Arc;

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigService;

use crate::{
    pages::helpers::{self, Keepalive, build_page_header, build_section},
    row::{SettingRow, SettingRowMsg},
};

#[allow(dead_code)]
pub struct BarButtonPage {
    rows: Vec<Controller<SettingRow>>,
    keepalives: Vec<Keepalive>,
}

#[derive(Debug)]
pub enum BarButtonPageMsg {
    ConfigChanged,
}

#[relm4::component(pub)]
impl SimpleComponent for BarButtonPage {
    type Init = Arc<ConfigService>;
    type Input = BarButtonPageMsg;
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

        content.append(&build_page_header("settings-page-bar-button"));

        let mut rows = Vec::new();

        rows.extend(build_section(
            content,
            "settings-section-style",
            vec![
                helpers::enum_select(&bar.button_variant, &mut keepalives),
                helpers::percentage(&bar.button_opacity, &mut keepalives),
                helpers::percentage(&bar.button_bg_opacity, &mut keepalives),
                helpers::enum_select(&bar.button_rounding, &mut keepalives),
                helpers::scale(&bar.button_gap, &mut keepalives),
                helpers::enum_select(&bar.button_icon_position, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-icons",
            vec![
                helpers::scale(&bar.button_icon_size, &mut keepalives),
                helpers::scale(&bar.button_icon_padding, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-labels",
            vec![
                helpers::scale(&bar.button_label_size, &mut keepalives),
                helpers::enum_select(&bar.button_label_weight, &mut keepalives),
                helpers::scale(&bar.button_label_padding, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-border",
            vec![
                helpers::enum_select(&bar.button_border_location, &mut keepalives),
                helpers::number_u8(&bar.button_border_width, &mut keepalives),
            ],
        ));

        rows.extend(build_section(
            content,
            "settings-section-group",
            vec![
                helpers::percentage(&bar.button_group_opacity, &mut keepalives),
                helpers::enum_select(&bar.button_group_rounding, &mut keepalives),
                helpers::spacing(&bar.button_group_padding, &mut keepalives),
                helpers::spacing(&bar.button_group_module_gap, &mut keepalives),
                helpers::enum_select(&bar.button_group_border_location, &mut keepalives),
                helpers::number_u8(&bar.button_group_border_width, &mut keepalives),
            ],
        ));

        let model = Self { rows, keepalives };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            BarButtonPageMsg::ConfigChanged => {
                for row in &self.rows {
                    row.emit(SettingRowMsg::Refresh);
                }
            }
        }
    }
}
