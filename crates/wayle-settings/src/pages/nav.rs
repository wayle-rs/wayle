//! Nav entry types that bundle a page's sidebar metadata with its content.
//! Each page module exports an `entry()` function returning one of these.

use super::helpers::PageSpec;

/// Sidebar item that navigates directly to a page.
pub(crate) struct LeafEntry {
    pub id: &'static str,
    pub i18n_key: &'static str,
    pub icon: &'static str,
    pub spec: PageSpec,
}
