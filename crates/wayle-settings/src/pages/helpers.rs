//! Factory functions for building SettingSpec instances from config
//! properties, plus layout helpers for page headers and grouped sections.

use std::any::Any;

use gtk4::prelude::*;
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use wayle_config::{
    ConfigProperty, EnumVariants,
    schemas::styling::{NormalizedF64, Percentage, ScaleFactor, Spacing},
};
use wayle_i18n::{t, t_attr};

use crate::{
    controls::{
        enum_select::EnumSelectControl,
        font::FontControl,
        number::{NumberControl, NumberInit},
        slider::{SliderControl, SliderInit},
        toggle::ToggleControl,
    },
    property_handle::PropertyHandle,
    row::{SettingRow, SettingRowInit},
};

pub type Keepalive = Box<dyn Any>;

pub struct SettingSpec {
    pub i18n_key: Option<&'static str>,
    pub handle: PropertyHandle,
    pub control: gtk4::Widget,
}

pub fn toggle(property: &ConfigProperty<bool>, controls: &mut Vec<Keepalive>) -> SettingSpec {
    let controller = ToggleControl::builder().launch(property.clone()).detach();

    let widget = controller.widget().clone();
    controls.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
    }
}

pub fn enum_select<E>(property: &ConfigProperty<E>, controls: &mut Vec<Keepalive>) -> SettingSpec
where
    E: EnumVariants
        + Clone
        + Send
        + Sync
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + 'static,
{
    let controller = EnumSelectControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();
    controls.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| {
            serde_json::to_string(value)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned()
        }),
        control: widget.upcast(),
    }
}

pub(crate) fn spacing(
    property: &ConfigProperty<Spacing>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: Spacing::MIN as f64,
            range_max: 500.0,
            step: 0.5,
            digits: 2,
            to_f64: |spacing| spacing.value() as f64,
            from_f64: |value| Spacing::new(value as f32),
        })
        .detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| format!("{}", value.value())),
        control: widget.upcast(),
    }
}

pub(crate) fn number_u8(
    property: &ConfigProperty<u8>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: f64::from(u8::MIN),
            range_max: f64::from(u8::MAX),
            step: 1.0,
            digits: 0,
            to_f64: |value| f64::from(*value),
            from_f64: |value| value.round() as u8,
        })
        .detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
    }
}

pub(crate) fn percentage(
    property: &ConfigProperty<Percentage>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = SliderControl::builder()
        .launch(SliderInit {
            property: property.clone(),
            range_min: Percentage::MIN as f64,
            range_max: Percentage::MAX as f64,
            to_slider: |pct| pct.value() as f64,
            from_slider: |value| Percentage::new(value.round() as u8),
            format_label: |value| format!("{value:.0}%"),
        })
        .detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |pct| format!("{}%", pct.value())),
        control: widget.upcast(),
    }
}

pub(crate) fn scale(
    property: &ConfigProperty<ScaleFactor>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = SliderControl::builder()
        .launch(SliderInit {
            property: property.clone(),
            range_min: ScaleFactor::MIN as f64,
            range_max: ScaleFactor::MAX as f64,
            to_slider: |sf| sf.value() as f64,
            from_slider: |value| ScaleFactor::new(value as f32),
            format_label: |value| format!("{value:.2}x"),
        })
        .detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |sf| format!("{:.2}x", sf.value())),
        control: widget.upcast(),
    }
}

pub(crate) fn normalized(
    property: &ConfigProperty<NormalizedF64>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = SliderControl::builder()
        .launch(SliderInit {
            property: property.clone(),
            range_min: NormalizedF64::MIN,
            range_max: NormalizedF64::MAX,
            to_slider: |nf| nf.value(),
            from_slider: NormalizedF64::new,
            format_label: |value| format!("{value:.2}"),
        })
        .detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |nf| format!("{:.2}", nf.value())),
        control: widget.upcast(),
    }
}

pub(crate) fn font(
    property: &ConfigProperty<String>,
    keepalives: &mut Vec<Keepalive>,
) -> SettingSpec {
    let controller = FontControl::builder().launch(property.clone()).detach();

    let widget = controller.widget().clone();
    keepalives.push(Box::new(controller));

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
    }
}

pub fn build_page_header(title_key: &str) -> gtk4::Box {
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

pub fn build_section(
    parent: &gtk4::Box,
    title_key: &str,
    items: Vec<SettingSpec>,
) -> Vec<Controller<SettingRow>> {
    let section = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    section.add_css_class("settings-section");

    let title = t(title_key);
    let section_title = gtk4::Label::builder()
        .label(&title)
        .halign(gtk4::Align::Start)
        .build();
    section_title.add_css_class("settings-section-title");
    section.append(&section_title);

    let group = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .build();
    group.add_css_class("settings-group");

    let mut controllers = Vec::new();

    for entry in items {
        let key = entry.i18n_key.unwrap_or("missing-i18n-key");

        let row = SettingRow::builder()
            .launch(SettingRowInit {
                i18n_key: key,
                handle: entry.handle,
                control_widget: Some(entry.control),
            })
            .detach();

        group.append(row.widget());
        controllers.push(row);
    }

    section.append(&group);
    parent.append(&section);

    controllers
}
