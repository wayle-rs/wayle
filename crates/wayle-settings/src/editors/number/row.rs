use std::fmt;

use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, schemas::styling::Spacing};

use crate::{
    editors::number::{NumberControl, NumberInit},
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

const U64_DISPLAY_MAX: f64 = 1_000_000.0;

pub(crate) fn spacing(property: &ConfigProperty<Spacing>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| format!("{}", value.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn number_u8(property: &ConfigProperty<u8>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn number_u32(property: &ConfigProperty<u32>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn number_u64(property: &ConfigProperty<u64>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn number_f64(
    property: &ConfigProperty<f64>,
    range_min: f64,
    range_max: f64,
    step: f64,
    digits: u32,
) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| format!("{value:.2}")),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}

pub(crate) fn number_newtype<T>(
    property: &ConfigProperty<T>,
    range_min: f64,
    range_max: f64,
    step: f64,
    digits: u32,
    to_f64: fn(&T) -> f64,
    from_f64: fn(f64) -> T,
) -> SettingRowInit
where
    T: Clone + Send + Sync + PartialEq + fmt::Display + 'static,
{
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min,
            range_max,
            step,
            digits,
            to_f64,
            from_f64,
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
