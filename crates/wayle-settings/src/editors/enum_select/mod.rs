//! Dropdown control for config enums that derive `EnumVariants`.

mod row;
pub(crate) use row::*;

use gtk4::prelude::*;
use relm4::prelude::*;
use serde::{Deserialize, de::value::StrDeserializer};
use wayle_config::{ConfigProperty, EnumVariant, EnumVariants};
use wayle_i18n::t;

use super::{ControlOutput, spawn_property_watcher};

pub(crate) struct EnumSelectControl<E: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<E>,
    selected: u32,
    dropdown: gtk4::DropDown,
    handler_id: gtk4::glib::SignalHandlerId,
}

#[derive(Debug)]
pub(crate) enum EnumSelectMsg {
    Selected(u32),
    Refresh,
}

impl<E> SimpleComponent for EnumSelectControl<E>
where
    E: EnumVariants + Clone + Send + Sync + PartialEq + for<'de> Deserialize<'de> + 'static,
{
    type Init = ConfigProperty<E>;
    type Input = EnumSelectMsg;
    type Output = ControlOutput;
    type Root = gtk4::Box;
    type Widgets = ();

    fn init_root() -> Self::Root {
        gtk4::Box::builder()
            .hexpand(false)
            .valign(gtk4::Align::Center)
            .build()
    }

    fn init(
        property: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let variants = E::variants();

        let labels: Vec<String> = variants
            .iter()
            .map(|variant| {
                let resolved = t(variant.fluent_key);

                if resolved == variant.fluent_key {
                    variant.value.to_owned()
                } else {
                    resolved
                }
            })
            .collect();

        let string_list =
            gtk4::StringList::new(&labels.iter().map(String::as_str).collect::<Vec<_>>());

        let current_index = variant_index_of(&property.get(), variants);

        let dropdown = gtk4::DropDown::new(Some(string_list), gtk4::Expression::NONE);
        dropdown.set_selected(current_index);
        dropdown.set_cursor_from_name(Some("pointer"));

        if let Some(popover) = dropdown
            .last_child()
            .and_then(|child| child.downcast::<gtk4::Popover>().ok())
        {
            popover.set_halign(gtk4::Align::Center);
        }

        let input_sender = sender.input_sender().clone();

        let handler_id = dropdown.connect_selected_notify(move |dropdown| {
            let _ = input_sender.send(EnumSelectMsg::Selected(dropdown.selected()));
        });

        let watcher_sender = sender.input_sender().clone();
        spawn_property_watcher(&property, move || {
            let _ = watcher_sender.send(EnumSelectMsg::Refresh);
        });

        root.append(&dropdown);

        let model = Self {
            property,
            selected: current_index,
            dropdown,
            handler_id,
        };

        ComponentParts { model, widgets: () }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            EnumSelectMsg::Selected(index) => {
                self.selected = index;

                if let Some(variant) = E::variants().get(index as usize)
                    && let Some(value) = enum_from_serde_value(variant.value)
                {
                    self.property.set(value);
                }

                let _ = sender.output(ControlOutput::ValueChanged);
            }

            EnumSelectMsg::Refresh => {
                let index = variant_index_of(&self.property.get(), E::variants());
                self.selected = index;

                self.dropdown.block_signal(&self.handler_id);
                self.dropdown.set_selected(index);
                self.dropdown.unblock_signal(&self.handler_id);
            }
        }
    }
}

fn variant_index_of<E>(current: &E, variants: &[EnumVariant]) -> u32
where
    E: for<'de> Deserialize<'de> + PartialEq,
{
    variants
        .iter()
        .position(|variant| enum_from_serde_value::<E>(variant.value).as_ref() == Some(current))
        .unwrap_or(0) as u32
}

fn enum_from_serde_value<E: for<'de> Deserialize<'de>>(value: &str) -> Option<E> {
    let deserializer: StrDeserializer<'_, serde::de::value::Error> = StrDeserializer::new(value);
    E::deserialize(deserializer).ok()
}
