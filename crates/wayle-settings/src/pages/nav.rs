//! Nav entry types that bundle a page's sidebar metadata with its content.
//! Each page module exports an `entry()` function returning one of these.

use super::helpers::PageSpec;

/// Top-level sidebar item that navigates directly to a page.
pub(crate) struct LeafEntry {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub icon: &'static str,
    pub spec: PageSpec,
}

/// Top-level sidebar item that expands to show child pages.
pub(crate) struct GroupEntry {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub icon: &'static str,
    pub children: Vec<ChildEntry>,
}

/// A page nested under a group in the sidebar.
pub(crate) struct ChildEntry {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub spec: PageSpec,
}
