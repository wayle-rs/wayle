//! Page layout builders: page headers and section/row assembly.

use relm4::{gtk, gtk::prelude::*, prelude::*};
use wayle_i18n::{t, t_attr};

use super::spec::SectionSpec;
use crate::row::SettingRow;

pub(crate) fn build_page_header(title_key: &str) -> gtk::Box {
    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    header.add_css_class("settings-page-header");

    let breadcrumb = t_attr(title_key, "breadcrumb");
    let crumb = gtk::Label::builder()
        .label(&breadcrumb)
        .halign(gtk::Align::Start)
        .build();
    crumb.add_css_class("settings-breadcrumb");

    let title = t(title_key);
    let title_label = gtk::Label::builder()
        .label(&title)
        .halign(gtk::Align::Start)
        .build();
    title_label.add_css_class("settings-page-title");

    header.append(&crumb);
    header.append(&title_label);

    header
}

pub(crate) fn build_sections(
    parent: &gtk::Box,
    sections: Vec<SectionSpec>,
) -> Vec<Controller<SettingRow>> {
    let mut rows = Vec::new();

    for section in sections {
        let section_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        section_box.add_css_class("settings-section");

        let title = t(section.title_key);
        let section_title = gtk::Label::builder()
            .label(&title)
            .halign(gtk::Align::Start)
            .build();
        section_title.add_css_class("settings-section-title");
        section_box.append(&section_title);

        let group = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();
        group.add_css_class("settings-group");

        for entry in section.items {
            let row = SettingRow::builder().launch(entry).detach();
            group.append(row.widget());
            rows.push(row);
        }

        section_box.append(&group);
        parent.append(&section_box);
    }

    rows
}
