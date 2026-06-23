use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use crate::{
    editors::{
        make_dirty_badge,
        vec_string::{VecStringControl, VecStringInit},
    },
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn vec_string(property: &ConfigProperty<Vec<String>>) -> SettingRowInit {
    let badge = make_dirty_badge();

    let controller = VecStringControl::builder()
        .launch(VecStringInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &Vec<String>| {
            VecStringControl::to_entry_text(value)
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
        unit: None,
    }
}
