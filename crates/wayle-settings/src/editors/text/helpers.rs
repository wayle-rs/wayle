//! `TextLike` impls for the domain value types that the text editor accepts.

use wayle_config::{
    ClickAction,
    schemas::{modules::PopupMonitor, osd::OsdMonitor},
};

use super::TextLike;

impl TextLike for String {
    fn to_entry_text(&self) -> String {
        self.clone()
    }

    fn from_entry_text(text: &str) -> Self {
        text.to_string()
    }
}

impl TextLike for Option<String> {
    fn to_entry_text(&self) -> String {
        self.as_deref().unwrap_or_default().to_owned()
    }

    fn from_entry_text(text: &str) -> Self {
        if text.is_empty() {
            return None;
        }
        Some(text.to_string())
    }
}

macro_rules! impl_monitor_text_like {
    ($type:ty) => {
        impl TextLike for $type {
            fn to_entry_text(&self) -> String {
                match self {
                    Self::Primary => String::from("primary"),
                    Self::Connector(name) => name.clone(),
                }
            }

            fn from_entry_text(text: &str) -> Self {
                if text.eq_ignore_ascii_case("primary") || text.is_empty() {
                    return Self::Primary;
                }
                Self::Connector(text.to_owned())
            }
        }
    };
}

impl_monitor_text_like!(OsdMonitor);
impl_monitor_text_like!(PopupMonitor);

impl TextLike for ClickAction {
    fn to_entry_text(&self) -> String {
        match self {
            Self::None => String::new(),
            Self::Dropdown(name) => format!("dropdown:{name}"),
            Self::Shell(cmd) => cmd.clone(),
        }
    }

    fn from_entry_text(text: &str) -> Self {
        if text.is_empty() {
            return Self::None;
        }

        match text.strip_prefix("dropdown:") {
            Some(name) => Self::Dropdown(name.to_owned()),
            None => Self::Shell(text.to_owned()),
        }
    }
}
