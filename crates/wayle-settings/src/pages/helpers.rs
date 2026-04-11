//! Factory functions for building SettingSpec instances from config
//! properties, plus layout helpers for page headers and grouped sections.

use std::any::Any;

use gtk4::prelude::*;
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use wayle_config::{
    ConfigProperty, EnumVariants,
    schemas::{
        bar::BarLayout,
        modules::CustomModuleDefinition,
        styling::{
            ColorValue, HexColor, NormalizedF64, Percentage, ScaleFactor, SignedNormalizedF64,
            Spacing,
        },
        wallpaper::MonitorWallpaperConfig,
    },
};
use wayle_i18n::{t, t_attr};

use crate::{
    controls::{
        bar_layout::{BarLayoutControl, BarLayoutInit},
        color::ColorControl,
        color_value::ColorValueControl,
        enum_select::EnumSelectControl,
        file_picker::{FilePickerControl, FilePickerInit},
        font::FontControl,
        monitor_wallpaper::MonitorWallpaperControl,
        number::{NumberControl, NumberInit},
        slider::{SliderControl, SliderInit},
        text::{TextControl, TextInit},
        toggle::ToggleControl,
        toml_editor::{TomlEditorControl, TomlEditorInit, helpers::serialize_with_key},
    },
    property_handle::PropertyHandle,
    row::{SettingRow, SettingRowInit},
};

pub(crate) type Keepalive = Box<dyn Any>;

const U64_DISPLAY_MAX: f64 = 1_000_000.0;

pub(crate) struct SettingSpec {
    pub i18n_key: Option<&'static str>,
    pub handle: PropertyHandle,
    pub control: gtk4::Widget,
    pub keepalive: Keepalive,
    pub full_width: bool,
    pub dirty_badge: Option<gtk4::Label>,
}

pub(crate) struct SectionSpec {
    pub title_key: &'static str,
    pub items: Vec<SettingSpec>,
}

pub(crate) struct PageSpec {
    pub header_key: &'static str,
    pub sections: Vec<SectionSpec>,
}

pub(crate) fn page_spec(header_key: &'static str, sections: Vec<SectionSpec>) -> PageSpec {
    PageSpec {
        header_key,
        sections,
    }
}

pub(crate) fn toggle(property: &ConfigProperty<bool>) -> SettingSpec {
    let controller = ToggleControl::builder().launch(property.clone()).detach();
    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn enum_select<E>(property: &ConfigProperty<E>) -> SettingSpec
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

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| {
            serde_json::to_string(value)
                .unwrap_or_default()
                .trim_matches('"')
                .to_owned()
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn spacing(property: &ConfigProperty<Spacing>) -> SettingSpec {
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

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| format!("{}", value.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn number_u8(property: &ConfigProperty<u8>) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: f64::from(u8::MIN),
            range_max: f64::from(u8::MAX),
            step: 1.0,
            digits: 0,
            to_f64: |value| f64::from(*value),
            from_f64: |value| value.round().clamp(f64::from(u8::MIN), f64::from(u8::MAX)) as u8,
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn percentage(property: &ConfigProperty<Percentage>) -> SettingSpec {
    let controller = SliderControl::builder()
        .launch(SliderInit {
            property: property.clone(),
            range_min: Percentage::MIN as f64,
            range_max: Percentage::MAX as f64,
            to_slider: |pct| pct.value() as f64,
            from_slider: |value| Percentage::new(value.round().clamp(0.0, 100.0) as u8),
            format_label: |value| format!("{value:.0}%"),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |pct| format!("{}%", pct.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn scale(property: &ConfigProperty<ScaleFactor>) -> SettingSpec {
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

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |sf| format!("{:.2}x", sf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn normalized(property: &ConfigProperty<NormalizedF64>) -> SettingSpec {
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

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |nf| format!("{:.2}", nf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn font(property: &ConfigProperty<String>) -> SettingSpec {
    let controller = FontControl::builder().launch(property.clone()).detach();
    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn text(property: &ConfigProperty<String>) -> SettingSpec {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
    }
}

pub(crate) fn optional_text(property: &ConfigProperty<Option<String>>) -> SettingSpec {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &Option<String>| {
            value.clone().unwrap_or_default()
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
    }
}

pub(crate) fn color(property: &ConfigProperty<HexColor>) -> SettingSpec {
    let controller = ColorControl::builder().launch(property.clone()).detach();
    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn color_value(property: &ConfigProperty<ColorValue>) -> SettingSpec {
    let controller = ColorValueControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| match value {
            ColorValue::Auto => "auto".to_owned(),
            ColorValue::Transparent => "transparent".to_owned(),
            ColorValue::Custom(hex) => hex.to_string(),
            ColorValue::Token(token) => token
                .as_str()
                .strip_prefix("--")
                .unwrap_or(token.as_str())
                .to_owned(),
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn file_path(property: &ConfigProperty<String>) -> SettingSpec {
    let badge = make_dirty_badge();

    let controller = FilePickerControl::builder()
        .launch(FilePickerInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
    }
}

pub(crate) fn number_u32(property: &ConfigProperty<u32>) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: f64::from(u32::MIN),
            range_max: f64::from(u32::MAX),
            step: 1.0,
            digits: 0,
            to_f64: |value| f64::from(*value),
            from_f64: |value| {
                value
                    .round()
                    .clamp(f64::from(u32::MIN), f64::from(u32::MAX)) as u32
            },
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn number_u64(property: &ConfigProperty<u64>) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: 0.0,
            range_max: U64_DISPLAY_MAX,
            step: 1.0,
            digits: 0,
            to_f64: |value| *value as f64,
            from_f64: |value| value.round().clamp(0.0, U64_DISPLAY_MAX) as u64,
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn number_f64(
    property: &ConfigProperty<f64>,
    range_min: f64,
    range_max: f64,
    step: f64,
    digits: u32,
) -> SettingSpec {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min,
            range_max,
            step,
            digits,
            to_f64: |value| *value,
            from_f64: |value| value,
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| format!("{value:.2}")),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn signed_normalized(property: &ConfigProperty<SignedNormalizedF64>) -> SettingSpec {
    let controller = SliderControl::builder()
        .launch(SliderInit {
            property: property.clone(),
            range_min: -1.0,
            range_max: 1.0,
            to_slider: |snf| snf.value(),
            from_slider: SignedNormalizedF64::new,
            format_label: |value| format!("{value:.2}"),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |snf| format!("{:.2}", snf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
    }
}

pub(crate) fn toml_editor<T>(property: &ConfigProperty<T>, key: &'static str) -> SettingSpec
where
    T: Clone + Send + Sync + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static,
{
    let badge = make_dirty_badge();

    let controller = TomlEditorControl::builder()
        .launch(TomlEditorInit {
            property: property.clone(),
            key,
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, move |value| serialize_with_key(value, key)),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: Some(badge),
    }
}

pub(crate) fn bar_layout(
    property: &ConfigProperty<Vec<BarLayout>>,
    custom_modules: &ConfigProperty<Vec<CustomModuleDefinition>>,
) -> SettingSpec {
    let controller = BarLayoutControl::builder()
        .launch(BarLayoutInit {
            property: property.clone(),
            custom_modules: custom_modules.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |layouts| layouts.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
    }
}

pub(crate) fn monitor_wallpaper(
    property: &ConfigProperty<Vec<MonitorWallpaperConfig>>,
) -> SettingSpec {
    let controller = MonitorWallpaperControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |monitors| monitors.len().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
    }
}

fn make_dirty_badge() -> gtk4::Label {
    let badge = gtk4::Label::new(Some(&t("settings-source-unsaved")));

    badge.add_css_class("badge-subtle");
    badge.add_css_class("warning");

    badge.set_visible(false);
    badge.set_valign(gtk4::Align::Center);
    badge.set_halign(gtk4::Align::Start);

    badge
}

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
