use gtk4::prelude::*;
use relm4::prelude::*;
use wayle_config::ConfigProperty;

use super::make_dirty_badge;
use crate::{
    editors::text::{TextControl, TextInit, TextLike},
    pages::helpers::types::SettingSpec,
    property_handle::PropertyHandle,
    row::RowBehavior,
};

pub(crate) fn text(property: &ConfigProperty<String>) -> SettingSpec {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
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

pub(crate) fn text_like<T: TextLike>(property: &ConfigProperty<T>) -> SettingSpec {
    let badge = make_dirty_badge();

    let controller = TextControl::builder()
        .launch(TextInit {
            property: property.clone(),
            dirty_badge: badge.clone(),
        })
        .detach();

    let widget = controller.widget().clone();

    SettingSpec {
        i18n_key: property.i18n_key(),
        handle: PropertyHandle::new(property, |value: &T| value.to_entry_text()),
        control: widget.upcast(),
        keepalive: Box::new(controller),
        full_width: false,
        dirty_badge: Some(badge),
        behavior: RowBehavior::Setting,
    }
}
