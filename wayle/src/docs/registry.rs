//! Lookup over every schema registered with `wayle_config::register_module!`.
//!
//! Providers are collected by the `inventory` crate at link time. This module
//! exposes them in a stable, sorted order so the generator's output is
//! deterministic.

use crate::config::docs::{self, ConfigGroup, ModuleInfo, ModuleRegistration};

/// A registered module's metadata paired with the group layout the generator
/// should render.
pub struct ModuleEntry {
    /// Page metadata.
    pub info: ModuleInfo,

    /// Groups rendered as H2 sections, in order.
    pub groups: Vec<ConfigGroup>,
}

/// Static set of schemas the generator emits pages for.
pub struct ModuleRegistry;

impl ModuleRegistry {
    /// Every registered entry, sorted by name for deterministic output.
    pub fn entries() -> Vec<ModuleEntry> {
        let mut entries: Vec<ModuleEntry> = docs::inventory::iter::<ModuleRegistration>()
            .map(|registration| {
                let (info, groups) = (registration.build_entry)();
                ModuleEntry { info, groups }
            })
            .collect();

        entries.sort_by(|left, right| left.info.name.cmp(&right.info.name));
        entries
    }

    /// The entry whose name matches, or `None` if nothing is registered under
    /// that name.
    pub fn find(name: &str) -> Option<ModuleEntry> {
        Self::entries()
            .into_iter()
            .find(|entry| entry.info.name == name)
    }

    /// Sorted names of every registered entry.
    pub fn names() -> Vec<String> {
        Self::entries()
            .into_iter()
            .map(|entry| entry.info.name)
            .collect()
    }
}
