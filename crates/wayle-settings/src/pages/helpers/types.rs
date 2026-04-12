//! Spec types that describe page structure: pages, sections, and individual settings.

use std::any::Any;

use crate::{property_handle::PropertyHandle, row::RowBehavior};

pub(crate) type Keepalive = Box<dyn Any>;

pub(crate) struct SettingSpec {
    pub i18n_key: Option<&'static str>,
    pub handle: PropertyHandle,
    pub control: gtk4::Widget,
    pub keepalive: Keepalive,
    pub full_width: bool,
    pub dirty_badge: Option<gtk4::Label>,
    pub behavior: RowBehavior,
}

pub(crate) struct SectionSpec {
    pub title_key: &'static str,
    pub items: Vec<SettingSpec>,
}

pub(crate) struct PageSpec {
    pub header_key: &'static str,
    pub sections: Vec<SectionSpec>,
}

pub(crate) fn page_spec(header_key: &'static str, sections: Vec<SectionSpec>) -> PageSpec {
    PageSpec {
        header_key,
        sections,
    }
}
