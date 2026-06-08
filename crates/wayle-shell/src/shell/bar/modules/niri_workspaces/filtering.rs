//! Filter and order the workspace list for display.
//!
//! Empty workspaces (no windows, not active) are hidden by default.
//! `min-workspace-count` raises a numeric-name ceiling: a hidden
//! workspace is included if its name parses to an integer at or below
//! that ceiling. Workspaces with non-numeric names are unaffected. No
//! placeholder workspaces are ever fabricated.

use std::collections::HashMap;

use super::helpers;

/// Plain-data view of one workspace, used as input to [`collect_displayed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceSnapshot {
    pub id: u64,
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub is_urgent: bool,
    pub is_active: bool,
    pub is_focused: bool,
    pub has_windows: bool,
}

/// Inputs that govern filtering, ordering, and padding.
#[derive(Debug, Clone)]
pub(super) struct FilterContext<'a> {
    pub monitor_specific: bool,
    pub bar_monitor: Option<&'a str>,
    pub hide_trailing_empty: bool,
    pub ignore_patterns: &'a [String],
}

pub(super) fn collect_displayed(
    snapshots: Vec<WorkspaceSnapshot>,
    ctx: &FilterContext<'_>,
) -> Vec<WorkspaceSnapshot> {
    let trailing_empty_ids = if ctx.hide_trailing_empty {
        compute_trailing_empties(&snapshots)
    } else {
        Vec::new()
    };

    let mut workspaces: Vec<WorkspaceSnapshot> = snapshots
        .into_iter()
        .filter(|snapshot| visible_on_monitor(snapshot, ctx))
        .filter(|snapshot| {
            !helpers::is_ignored(
                snapshot.name.as_deref(),
                snapshot.idx,
                snapshot.id,
                ctx.ignore_patterns,
            )
        })
        .filter(|snapshot| !trailing_empty_ids.contains(&snapshot.id))
        .collect();

    workspaces.sort_by_key(sort_key);
    workspaces
}

fn visible_on_monitor(snapshot: &WorkspaceSnapshot, ctx: &FilterContext<'_>) -> bool {
    if !ctx.monitor_specific {
        return true;
    }
    let Some(bar_monitor) = ctx.bar_monitor else {
        return true;
    };
    snapshot
        .output
        .as_deref()
        .is_some_and(|output| output == bar_monitor)
}

fn compute_trailing_empties(snapshots: &[WorkspaceSnapshot]) -> Vec<u64> {
    let mut last_per_output: HashMap<String, OutputTail> = HashMap::new();

    for snapshot in snapshots {
        let Some(output) = snapshot.output.clone() else {
            continue;
        };
        let candidate = OutputTail {
            idx: snapshot.idx,
            id: snapshot.id,
            is_empty: !snapshot.has_windows,
        };
        last_per_output
            .entry(output)
            .and_modify(|tail| {
                if candidate.idx > tail.idx {
                    *tail = candidate.clone();
                }
            })
            .or_insert(candidate);
    }

    last_per_output
        .into_values()
        .filter(|tail| tail.is_empty)
        .map(|tail| tail.id)
        .collect()
}

#[derive(Clone)]
struct OutputTail {
    idx: u8,
    id: u64,
    is_empty: bool,
}

fn sort_key(snapshot: &WorkspaceSnapshot) -> (String, u8) {
    let output = snapshot.output.clone().unwrap_or_default();
    (output, snapshot.idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn occupied(id: u64, idx: u8, output: &str) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            id,
            idx,
            name: None,
            output: Some(output.to_string()),
            is_urgent: false,
            is_active: false,
            is_focused: false,
            has_windows: true,
        }
    }

    fn named_empty(id: u64, idx: u8, output: &str, name: &str) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            id,
            idx,
            name: Some(name.to_string()),
            output: Some(output.to_string()),
            is_urgent: false,
            is_active: false,
            is_focused: false,
            has_windows: false,
        }
    }

    fn empty(id: u64, idx: u8, output: &str) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            id,
            idx,
            name: None,
            output: Some(output.to_string()),
            is_urgent: false,
            is_active: false,
            is_focused: false,
            has_windows: false,
        }
    }

    fn active_empty(id: u64, idx: u8, output: &str) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            id,
            idx,
            name: None,
            output: Some(output.to_string()),
            is_urgent: false,
            is_active: true,
            is_focused: true,
            has_windows: false,
        }
    }

    fn ctx_default<'a>() -> FilterContext<'a> {
        FilterContext {
            monitor_specific: false,
            bar_monitor: None,
            hide_trailing_empty: false,
            min_workspace_count: 0,
            ignore_patterns: &[],
        }
    }

    fn ids(displayed: &[WorkspaceSnapshot]) -> Vec<u64> {
        displayed.iter().map(|snapshot| snapshot.id).collect()
    }

    #[test]
    fn hides_empty_workspaces_by_default() {
        let snapshots = vec![
            occupied(1, 1, "DP-1"),
            empty(2, 2, "DP-1"),
            named_empty(3, 3, "DP-1", "web"),
        ];
        let displayed = collect_displayed(snapshots, &ctx_default());
        assert_eq!(ids(&displayed), vec![1]);
    }

    #[test]
    fn always_shows_active_even_when_empty() {
        let snapshots = vec![occupied(1, 1, "DP-1"), active_empty(2, 2, "DP-1")];
        let displayed = collect_displayed(snapshots, &ctx_default());
        assert_eq!(ids(&displayed), vec![1, 2]);
    }

    #[test]
    fn min_includes_hidden_workspaces_with_numeric_name_in_range() {
        let snapshots = vec![
            occupied(10, 1, "DP-1"),
            named_empty(20, 2, "DP-1", "2"),
            named_empty(30, 3, "DP-1", "3"),
        ];
        let ctx = FilterContext {
            min_workspace_count: 3,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![10, 20, 30]);
    }

    #[test]
    fn min_does_not_force_show_above_range() {
        let snapshots = vec![
            occupied(10, 1, "DP-1"),
            named_empty(20, 2, "DP-1", "2"),
            named_empty(99, 9, "DP-1", "9"),
        ];
        let ctx = FilterContext {
            min_workspace_count: 3,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![10, 20]);
    }

    #[test]
    fn min_includes_named_workspace_assigned_to_this_monitor_only() {
        let snapshots = vec![
            occupied(10, 1, "DP-1"),
            named_empty(20, 2, "DP-1", "2"),
            named_empty(30, 1, "DP-2", "3"),
        ];
        let ctx = FilterContext {
            monitor_specific: true,
            bar_monitor: Some("DP-1"),
            min_workspace_count: 5,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![10, 20]);
    }

    #[test]
    fn min_ignores_non_numeric_names() {
        let snapshots = vec![
            occupied(10, 1, "DP-1"),
            named_empty(20, 2, "DP-1", "web"),
            named_empty(30, 3, "DP-1", "term"),
        ];
        let ctx = FilterContext {
            min_workspace_count: 100,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![10]);
    }

    #[test]
    fn min_six_two_two_distribution_matches_hyprland() {
        let snapshots = vec![
            occupied(11, 1, "DP-1"),
            named_empty(12, 2, "DP-1", "2"),
            named_empty(13, 3, "DP-1", "3"),
            named_empty(14, 4, "DP-3", "4"),
            named_empty(15, 5, "DP-3", "5"),
            named_empty(16, 1, "DP-2", "6"),
            named_empty(17, 2, "DP-2", "7"),
            named_empty(18, 4, "DP-1", "8"),
            named_empty(19, 5, "DP-1", "9"),
            named_empty(20, 6, "DP-1", "10"),
        ];

        let ctx_dp1 = FilterContext {
            monitor_specific: true,
            bar_monitor: Some("DP-1"),
            min_workspace_count: 10,
            ..ctx_default()
        };
        let displayed_dp1 = collect_displayed(snapshots.clone(), &ctx_dp1);
        assert_eq!(ids(&displayed_dp1), vec![11, 12, 13, 18, 19, 20]);

        let ctx_dp2 = FilterContext {
            monitor_specific: true,
            bar_monitor: Some("DP-2"),
            min_workspace_count: 10,
            ..ctx_default()
        };
        let displayed_dp2 = collect_displayed(snapshots.clone(), &ctx_dp2);
        assert_eq!(ids(&displayed_dp2), vec![16, 17]);

        let ctx_dp3 = FilterContext {
            monitor_specific: true,
            bar_monitor: Some("DP-3"),
            min_workspace_count: 10,
            ..ctx_default()
        };
        let displayed_dp3 = collect_displayed(snapshots, &ctx_dp3);
        assert_eq!(ids(&displayed_dp3), vec![14, 15]);
    }

    #[test]
    fn hide_trailing_empty_runs_before_partition() {
        let snapshots = vec![
            occupied(1, 1, "DP-1"),
            occupied(2, 2, "DP-1"),
            empty(3, 3, "DP-1"),
        ];
        let ctx = FilterContext {
            hide_trailing_empty: true,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![1, 2]);
    }

    #[test]
    fn workspace_ignore_drops_by_name_glob() {
        let snapshots = vec![
            occupied(1, 1, "DP-1"),
            occupied(2, 2, "DP-1"),
            named_empty(3, 3, "DP-1", "scratch"),
        ];
        let patterns = vec![String::from("scratch")];
        let ctx = FilterContext {
            min_workspace_count: 3,
            ignore_patterns: &patterns,
            ..ctx_default()
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(ids(&displayed), vec![1, 2]);
    }

    #[test]
    fn ordering_default_is_by_output_then_idx() {
        let snapshots = vec![
            occupied(3, 2, "DP-2"),
            occupied(1, 2, "DP-1"),
            occupied(2, 1, "DP-2"),
        ];
        let displayed = collect_displayed(snapshots, &ctx_default());
        assert_eq!(ids(&displayed), vec![1, 2, 3]);
    }
}
