//! Dropdown control for config enums that derive `EnumVariants`.

use gtk4::prelude::*;
use relm4::prelude::*;
use serde::{Deserialize, Serialize};
use wayle_config::{ConfigProperty, EnumVariants};
use wayle_i18n::t;

use super::ControlOutput;

pub struct EnumSelectControl<E: Clone + Send + Sync + PartialEq + 'static> {
    property: ConfigProperty<E>,
    selected: u32,
}

#[derive(Debug)]
pub enum EnumSelectMsg {
    Selected(u32),
}

impl<E> SimpleComponent for EnumSelectControl<E>
where
    E: EnumVariants
        + Clone
        + Send
        + Sync
        + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>
        + 'static,
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

        if let Some(popover) = dropdown
            .last_child()
            .and_then(|child| child.downcast::<gtk4::Popover>().ok())
        {
            popover.set_halign(gtk4::Align::Center);
        }

        let input_sender = sender.input_sender().clone();

        dropdown.connect_selected_notify(move |dropdown| {
            let _ = input_sender.send(EnumSelectMsg::Selected(dropdown.selected()));
        });

        root.append(&dropdown);

        let model = Self {
            property,
            selected: current_index,
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
        }
    }
}

fn variant_index_of<E: Serialize>(current: &E, variants: &[wayle_config::EnumVariant]) -> u32 {
    let serialized = serde_json::to_string(current).unwrap_or_default();
    let current_value = serialized.trim_matches('"');

    variants
        .iter()
        .position(|variant| variant.value == current_value)
        .unwrap_or(0) as u32
}

fn enum_from_serde_value<E: for<'de> Deserialize<'de>>(value: &str) -> Option<E> {
    let json = format!("\"{value}\"");
    serde_json::from_str(&json).ok()
}
