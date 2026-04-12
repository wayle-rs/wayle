use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::{ConfigProperty, schemas::styling::ColorValue};

use crate::{
    editors::color_value::ColorValueControl, pages::spec::SettingSpec,
    property_handle::PropertyHandle, row::RowBehavior,
};

pub(crate) fn color_value(property: &ConfigProperty<ColorValue>) -> SettingSpec {
    let controller = ColorValueControl::builder()
        .launch(property.clone())
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value| match value {
            ColorValue::Auto => "auto".to_owned(),
            ColorValue::Transparent => "transparent".to_owned(),
            ColorValue::Custom(hex) => hex.to_string(),
            ColorValue::Token(token) => token
                .as_str()
                .strip_prefix("--")
                .unwrap_or(token.as_str())
                .to_owned(),
        }),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: None,
        behavior: RowBehavior::Setting,
    }
}
