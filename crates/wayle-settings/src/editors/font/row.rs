use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::ConfigProperty;

use crate::{
    editors::font::FontControl, pages::spec::SettingRowInit, property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn font(property: &ConfigProperty<String>) -> SettingRowInit {
    let controller = FontControl::builder().launch(property.clone()).detach();
    let widget = controller.widget().clone();

    SettingRowInit {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &String| value.clone()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
