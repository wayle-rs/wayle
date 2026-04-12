use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::make_dirty_badge;
use crate::{
    controls::file_picker::{FilePickerControl, FilePickerInit},
    pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

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
        behavior: RowBehavior::Setting,
    }
}
