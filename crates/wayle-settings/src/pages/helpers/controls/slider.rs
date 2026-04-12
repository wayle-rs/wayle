use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{
    ConfigProperty,
    schemas::styling::{NormalizedF64, Percentage, ScaleFactor, SignedNormalizedF64},
};

use crate::{
    editors::slider::{SliderControl, SliderInit},
    pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

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
        behavior: RowBehavior::Setting,
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
        behavior: RowBehavior::Setting,
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
        behavior: RowBehavior::Setting,
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
        behavior: RowBehavior::Setting,
    }
}
