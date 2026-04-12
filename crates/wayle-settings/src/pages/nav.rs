//! Nav entry types that bundle a page's sidebar metadata with its content.
//! Each page module exports an `entry()` function returning one of these.

use super::spec::PageSpec;

pub(crate) struct LeafEntry {
    pub(crate) id: &'static str,
    pub(crate) i18n_key: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) spec: PageSpec,
}
