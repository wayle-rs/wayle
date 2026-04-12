use relm4::{gtk::prelude::*, prelude::*};
use wayle_config::{ConfigProperty, schemas::styling::HexColor};

use crate::{
    editors::color::ColorControl, pages::spec::SettingRowInit, property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn color(property: &ConfigProperty<HexColor>) -> SettingRowInit {
    let controller = ColorControl::builder().launch(property.clone()).detach();
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
