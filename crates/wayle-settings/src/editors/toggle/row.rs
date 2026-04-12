use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use crate::{
    editors::toggle::ToggleControl, pages::spec::SettingRowInit, property_handle::PropertyHandle,
    row::RowBehavior,
};

/// Row with a switch bound to a boolean property.
pub(crate) fn toggle(property: &ConfigProperty<bool>) -> SettingRowInit {
    let controller = ToggleControl::builder().launch(property.clone()).detach();
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
