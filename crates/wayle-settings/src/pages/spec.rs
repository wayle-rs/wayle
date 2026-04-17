//! Spec types that describe page structure: pages, sections, and individual settings.

use std::any::Any;

use relm4::gtk;

use crate::{property_handle::PropertyHandle, row::RowBehavior};

pub(crate) type Keepalive = Box<dyn Any>;

pub(crate) struct SettingRowInit {
    pub(crate) i18n_key: Option<&'static str>,
    pub(crate) handle: PropertyHandle,
    pub(crate) control: gtk::Widget,
    pub(crate) keepalive: Keepalive,
    pub(crate) full_width: bool,
    pub(crate) dirty_badge: Option<gtk::Label>,
    pub(crate) behavior: RowBehavior,
    pub(crate) unit: Option<String>,
}

pub(crate) struct SectionSpec {
    pub(crate) title_key: &'static str,
    pub(crate) items: Vec<SettingRowInit>,
}

pub(crate) struct PageSpec {
    pub(crate) header_key: &'static str,
    pub(crate) sections: Vec<SectionSpec>,
}

pub(crate) fn page_spec(header_key: &'static str, sections: Vec<SectionSpec>) -> PageSpec {
    PageSpec {
        header_key,
        sections,
    }
}
