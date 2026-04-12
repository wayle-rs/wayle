use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use crate::{
    controls::toggle::ToggleControl, pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle, row::RowBehavior,
};

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
        behavior: RowBehavior::Setting,
    }
}
