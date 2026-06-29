use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use super::{AppPickerControl, AppPickerInit};
use crate::{
    editors::make_dirty_badge, pages::spec::SettingRowInit, property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn app_picker(property: &ConfigProperty<Vec<String>>) -> SettingRowInit {
    let badge = make_dirty_badge();

    let controller = AppPickerControl::builder()
        .launch(AppPickerInit {
            property: property.clone(),
            pinned_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |values: &Vec<String>| {
            if values.is_empty() {
                "None".to_string()
            } else {
                values.join(", ")
            }
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: true,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
        unit: None,
    }
}
