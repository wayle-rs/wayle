//! [`MangoTagButton`] private helpers: state-driven rendering decisions.

use std::{collections::HashSet, mem};

use relm4::gtk::{self, prelude::*};
use wayle_config::schemas::modules::{DisplayMode, UrgentMode};

use super::{MangoTagButton, MangoTagButtonInit};
use crate::shell::bar::modules::mango_workspaces::helpers;

const URGENT_CLASS: &str = "urgent";
const WORKSPACE_ICON_CSS: &str = "workspace-icon";
const WORKSPACE_ICON_EMPTY_CSS: &str = "workspace-icon-empty";

impl MangoTagButton {
    pub(super) fn show_label(&self) -> bool {
        let has_label = self.label.as_deref().is_some_and(|label| !label.is_empty());
        if !has_label {
            return false;
        }

        match self.display_mode {
            DisplayMode::Label => true,
            DisplayMode::Icon => self.icon.is_none(),
            DisplayMode::None => false,
        }
    }

    pub(super) fn show_icon(&self) -> bool {
        matches!(self.display_mode, DisplayMode::Icon) && self.icon.is_some()
    }

    pub(super) fn label_text(&self) -> &str {
        self.label.as_deref().unwrap_or("")
    }

    pub(super) fn orientation(&self) -> gtk::Orientation {
        if self.is_vertical {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        }
    }

    pub(super) fn content_halign(&self) -> gtk::Align {
        if self.is_vertical {
            gtk::Align::Fill
        } else {
            gtk::Align::Center
        }
    }

    pub(super) fn content_valign(&self) -> gtk::Align {
        if self.is_vertical {
            gtk::Align::Center
        } else {
            gtk::Align::Fill
        }
    }

    pub(super) fn icons_halign(&self) -> gtk::Align {
        if self.is_vertical {
            gtk::Align::Center
        } else {
            gtk::Align::Fill
        }
    }

    pub(super) fn show_divider(&self) -> bool {
        self.show_app_icons && !self.divider.is_empty() && (self.show_label() || self.show_icon())
    }

    pub(super) fn show_identity_row(&self) -> bool {
        self.show_label() || self.show_icon() || self.show_divider()
    }

    pub(super) fn populate_app_icons(
        &mut self,
        container: &gtk::Box,
        urgent_client_ids: &HashSet<u32>,
    ) {
        if self.app_icon_inits.is_empty() {
            let image = gtk::Image::builder()
                .icon_name(&self.empty_icon)
                .css_classes([WORKSPACE_ICON_CSS, WORKSPACE_ICON_EMPTY_CSS])
                .valign(gtk::Align::Center)
                .build();
            container.append(&image);
            return;
        }

        for init in mem::take(&mut self.app_icon_inits) {
            let image = gtk::Image::builder()
                .icon_name(&init.icon_name)
                .css_classes([WORKSPACE_ICON_CSS])
                .valign(gtk::Align::Center)
                .build();

            let is_urgent = init
                .client_ids
                .iter()
                .any(|client_id| urgent_client_ids.contains(client_id));
            if is_urgent {
                image.add_css_class(URGENT_CLASS);
            }

            container.append(&image);
        }
    }
}

pub(super) fn compute_css_classes(init: &MangoTagButtonInit) -> Vec<String> {
    let mut classes = vec![String::from("workspace")];

    let state = if init.is_active {
        "active"
    } else if init.has_clients {
        "occupied"
    } else {
        "empty"
    };
    classes.push(state.to_string());

    if init.is_urgent && init.urgent_show {
        classes.push(String::from("urgent"));
        if matches!(init.urgent_mode, UrgentMode::Application) {
            classes.push(String::from("urgent-application"));
        }
    }

    classes.push(init.active_indicator.css_class().to_string());

    if init.is_vertical {
        classes.push(String::from("vertical"));
    }

    classes.push(helpers::tag_css_class(init.index));

    classes
}
