use std::fmt;

use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::styling::{ScaleFactor, Spacing},
};
use wayle_i18n::t;

use crate::{
    editors::number::{NumberControl, NumberInit},
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

/// Largest `u64` that round-trips through `f64` without precision loss (2^53).
const U64_SPIN_MAX: f64 = 9_007_199_254_740_992.0;

/// Row with a numeric spin bound to a `Spacing` property, stepping in 0.5 pixel increments up to 500.
pub(crate) fn spacing(property: &ConfigProperty<Spacing>) -> SettingRowInit {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: Spacing::MIN as f64,
            range_max: 500.0,
            step: 0.5,
            digits: 2,
            to_f64: |spacing| spacing.value() as f64,
            from_f64: |value| {
                let clean = if value.is_finite() { value } else { 0.0 };
                Spacing::new(clean.clamp(Spacing::MIN as f64, 500.0) as f32)
            },
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| value.value().to_string()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: Some(String::from("rem")),
    }
}

/// Row with an integer spin covering the full `u8` range.
pub(crate) fn number_u8(property: &ConfigProperty<u8>) -> SettingRowInit {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: f64::from(u8::MIN),
            range_max: f64::from(u8::MAX),
            step: 1.0,
            digits: 0,
            to_f64: |value| f64::from(*value),
            from_f64: |value| {
                if !value.is_finite() {
                    return 0;
                }
                value.round().clamp(f64::from(u8::MIN), f64::from(u8::MAX)) as u8
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
        unit: None,
    }
}

/// Row with an integer spin covering the full `u32` range.
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
                if !value.is_finite() {
                    return 0;
                }
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
        unit: None,
    }
}

/// Row with an integer spin for a `u64` property, capped at 2^53 (f64 integer precision limit).
pub(crate) fn number_u64(property: &ConfigProperty<u64>) -> SettingRowInit {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: 0.0,
            range_max: U64_SPIN_MAX,
            step: 1.0,
            digits: 0,
            to_f64: |value| *value as f64,
            from_f64: |value| {
                if !value.is_finite() {
                    return 0;
                }
                value.round().clamp(0.0, U64_SPIN_MAX) as u64
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
        unit: None,
    }
}

/// Row with a numeric spin for an `f64` property, using caller-supplied range, step, and decimal digits.
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
            from_f64: |value| if value.is_finite() { value } else { 0.0 },
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
        unit: None,
    }
}

/// Row with a numeric spin bound to a `ScaleFactor`, stepping in 0.05 increments.
pub(crate) fn scale(property: &ConfigProperty<ScaleFactor>) -> SettingRowInit {
    let controller = NumberControl::builder()
        .launch(NumberInit {
            property: property.clone(),
            range_min: ScaleFactor::MIN as f64,
            range_max: ScaleFactor::MAX as f64,
            step: 0.05,
            digits: 2,
            to_f64: |sf| sf.value() as f64,
            from_f64: |value| {
                let clean = if value.is_finite() { value } else { 0.0 };
                ScaleFactor::new(
                    clean.clamp(ScaleFactor::MIN as f64, ScaleFactor::MAX as f64) as f32,
                )
            },
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |sf| format!("{:.2}x", sf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: Some(t("settings-unit-scale")),
    }
}

/// Row with a numeric spin for any newtype around a numeric value, using caller-supplied conversions and range.
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
        unit: None,
    }
}
