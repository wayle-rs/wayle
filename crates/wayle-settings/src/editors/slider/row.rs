use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{
    ConfigProperty,
    schemas::styling::{NormalizedF64, Percentage, SignedNormalizedF64},
};

use crate::{
    editors::slider::{SliderControl, SliderInit},
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

/// Row with a 0 to 100 slider formatted as a percentage.
pub(crate) fn percentage(property: &ConfigProperty<Percentage>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |pct| format!("{}%", pct.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: None,
    }
}

/// Row with a slider over the -1.0 to 1.0 range for a `SignedNormalizedF64` property.
pub(crate) fn signed_normalized(property: &ConfigProperty<SignedNormalizedF64>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |snf| format!("{:.2}", snf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: None,
    }
}

/// Row with a slider over the 0.0 to 1.0 range for a `NormalizedF64` property.
pub(crate) fn normalized(property: &ConfigProperty<NormalizedF64>) -> SettingRowInit {
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

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |nf| format!("{:.2}", nf.value())),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: None,
    }
}
