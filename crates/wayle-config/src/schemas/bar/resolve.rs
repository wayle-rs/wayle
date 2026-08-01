//! Resolving a monitor's effective bar layout from the configured list.
//!
//! A `[[bar.layout]]` entry may target a specific connector or the `"*"`
//! wildcard, and may `extends` another entry to inherit whole sections. This
//! turns that list + a connector into a single flattened [`BarLayout`], and is
//! shared by the shell (which builds the bar) and the IPC daemon (which
//! enumerates addressable dropdowns) so both resolve layouts identically.

use std::collections::HashSet;

use tracing::warn;

use super::BarLayout;

/// Finds the layout matching `connector` (exact match first, then `"*"` wildcard)
/// and resolves any `extends` chain into a single flattened layout.
pub fn find_layout(layouts: &[BarLayout], connector: &str) -> Option<BarLayout> {
    let mut visited = HashSet::new();

    if let Some(layout) = layouts
        .iter()
        .find(|candidate| candidate.monitor == connector)
    {
        return Some(merge_parent(layout, layouts, &mut visited));
    }

    if let Some(layout) = layouts.iter().find(|candidate| candidate.monitor == "*") {
        return Some(merge_parent(layout, layouts, &mut visited));
    }

    None
}

fn merge_parent(
    layout: &BarLayout,
    all_layouts: &[BarLayout],
    visited: &mut HashSet<String>,
) -> BarLayout {
    let mut resolved = layout.clone();

    let Some(ref extends_name) = layout.extends else {
        return resolved;
    };

    if !visited.insert(extends_name.clone()) {
        warn!(
            layout = %layout.monitor,
            extends = %extends_name,
            "circular extends detected, skipping parent"
        );
        return resolved;
    }

    let Some(parent) = all_layouts
        .iter()
        .find(|candidate| candidate.monitor == *extends_name)
    else {
        return resolved;
    };

    let parent_resolved = merge_parent(parent, all_layouts, visited);

    if resolved.left.is_empty() {
        resolved.left = parent_resolved.left;
    }
    if resolved.center.is_empty() {
        resolved.center = parent_resolved.center;
    }
    if resolved.right.is_empty() {
        resolved.right = parent_resolved.right;
    }

    resolved
}
