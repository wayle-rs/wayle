use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use crate::{
    editors::{
        file_picker::{FilePickerControl, FilePickerInit},
        make_dirty_badge,
    },
    pages::spec::SettingRowInit,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn file_path(property: &ConfigProperty<String>) -> SettingRowInit {
    let badge = make_dirty_badge();

    let controller = FilePickerControl::builder()
        .launch(FilePickerInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
    }
}
