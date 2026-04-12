//! Page layout builders: page headers and section/row assembly.

use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_i18n::{t, t_attr};

use super::types::{Keepalive, SectionSpec};
use crate::row::{SettingRow, SettingRowInit};

pub(crate) fn build_page_header(title_key: &str) -> gtk4::Box {
    let header = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    header.add_css_class("settings-page-header");

    let breadcrumb = t_attr(title_key, "breadcrumb");
    let crumb = gtk4::Label::builder()
        .label(&breadcrumb)
        .halign(gtk4::Align::Start)
        .build();
    crumb.add_css_class("settings-breadcrumb");

    let title = t(title_key);
    let title_label = gtk4::Label::builder()
        .label(&title)
        .halign(gtk4::Align::Start)
        .build();
    title_label.add_css_class("settings-page-title");

    header.append(&crumb);
    header.append(&title_label);

    header
}

pub(crate) fn build_sections(
    parent: &gtk4::Box,
    sections: Vec<SectionSpec>,
) -> (Vec<Controller<SettingRow>>, Vec<Keepalive>) {
    let mut rows = Vec::new();
    let mut keepalives = Vec::new();

    for section in sections {
        let section_box = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        section_box.add_css_class("settings-section");

        let title = t(section.title_key);
        let section_title = gtk4::Label::builder()
            .label(&title)
            .halign(gtk4::Align::Start)
            .build();
        section_title.add_css_class("settings-section-title");
        section_box.append(&section_title);

        let group = gtk4::Box::builder()
            .orientation(gtk4::Orientation::Vertical)
            .build();
        group.add_css_class("settings-group");

        for entry in section.items {
            let key = entry.i18n_key.unwrap_or("missing-i18n-key");
            keepalives.push(entry.keepalive);

            let row = SettingRow::builder()
                .launch(SettingRowInit {
                    i18n_key: key,
                    handle: entry.handle,
                    control_widget: Some(entry.control),
                    full_width: entry.full_width,
                    dirty_badge: entry.dirty_badge,
                    behavior: entry.behavior,
                })
                .detach();

            group.append(row.widget());
            rows.push(row);
        }

        section_box.append(&group);
        parent.append(&section_box);
    }

    (rows, keepalives)
}
