use std::collections::HashSet;

use relm4::{gtk, gtk::prelude::*};
use wayle_config::schemas::modules::DisplayMode;
use wayle_hyprland::{Address, WorkspaceId};

use super::{
    AppIcon, WORKSPACE_CUSTOM_ICON_CSS, WORKSPACE_ICON_CSS, WORKSPACE_ICON_EMPTY_CSS,
    WORKSPACE_LABEL_CSS, WorkspaceButton,
};
use crate::shell::bar::modules::hyprland_workspaces::helpers::{
    collect_button_css_classes, format_workspace_label, should_show_divider,
};

impl WorkspaceButton {
    pub fn id(&self) -> WorkspaceId {
        self.id
    }

    pub(super) fn apply_urgency(&mut self, is_urgent: bool, urgent_addresses: &HashSet<Address>) {
        let per_icon = !urgent_addresses.is_empty() && !self.app_icons.is_empty();

        if per_icon {
            self.apply_per_icon_urgency(urgent_addresses);
            self.is_urgent = false;
            return;
        }

        self.clear_icon_urgency();
        self.is_urgent = is_urgent;
    }

    fn apply_per_icon_urgency(&self, urgent_addresses: &HashSet<Address>) {
        for app_icon in &self.app_icons {
            let is_urgent = app_icon
                .addresses
                .iter()
                .any(|addr| urgent_addresses.contains(addr));
            if is_urgent {
                app_icon.widget.add_css_class("urgent");
            } else {
                app_icon.widget.remove_css_class("urgent");
            }
        }
    }

    fn clear_icon_urgency(&self) {
        for app_icon in &self.app_icons {
            app_icon.widget.remove_css_class("urgent");
        }
    }

    pub(super) fn current_css_classes(&self) -> Vec<&str> {
        let mut classes = collect_button_css_classes(
            &self.static_classes,
            &self.css_id_class,
            self.state,
            self.is_urgent,
        );
        classes.push(&self.css_name_class);
        classes
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
        should_show_divider(self.show_app_icons, &self.divider, self.display_mode)
    }

    pub(super) fn populate_identity(&self, container: &gtk::Box) {
        match self.display_mode {
            DisplayMode::Label => {
                let label_text = format_workspace_label(
                    self.display_id,
                    self.id,
                    &self.name,
                    self.label_use_name,
                );
                let label = gtk::Label::builder()
                    .label(&label_text)
                    .css_classes([WORKSPACE_LABEL_CSS])
                    .valign(gtk::Align::Center)
                    .build();
                container.append(&label);
            }
            DisplayMode::Icon => {
                let Some(ref icon_name) = self.mapped_icon else {
                    let label_text = format_workspace_label(
                        self.display_id,
                        self.id,
                        &self.name,
                        self.label_use_name,
                    );
                    let label = gtk::Label::builder()
                        .label(&label_text)
                        .css_classes([WORKSPACE_LABEL_CSS])
                        .valign(gtk::Align::Center)
                        .build();
                    container.append(&label);
                    return;
                };
                let image = gtk::Image::builder()
                    .icon_name(icon_name)
                    .css_classes([WORKSPACE_CUSTOM_ICON_CSS])
                    .valign(gtk::Align::Center)
                    .build();
                container.append(&image);
            }
            DisplayMode::None => {}
        }
    }

    pub(super) fn populate_app_icons(
        &mut self,
        container: &gtk::Box,
        urgent_addresses: &HashSet<Address>,
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

        for init in self.app_icon_inits.drain(..) {
            let image = gtk::Image::builder()
                .icon_name(&init.icon_name)
                .css_classes([WORKSPACE_ICON_CSS])
                .valign(gtk::Align::Center)
                .build();
            let is_urgent = init
                .addresses
                .iter()
                .any(|addr| urgent_addresses.contains(addr));
            if is_urgent {
                image.add_css_class("urgent");
            }
            container.append(&image);
            self.app_icons.push(AppIcon {
                addresses: init.addresses,
                widget: image,
            });
        }
    }
}
