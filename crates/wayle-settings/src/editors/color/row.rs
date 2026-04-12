use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, schemas::styling::HexColor};

use crate::{
    editors::color::ColorControl, pages::spec::SettingSpec,
    property_handle::PropertyHandle, row::RowBehavior,
};

pub(crate) fn color(property: &ConfigProperty<HexColor>) -> SettingSpec {
    let controller = ColorControl::builder().launch(property.clone()).detach();
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
