use std::collections::{HashMap, HashSet};

use wayle_hyprland::WorkspaceId;

use super::helpers::matches_ignore_patterns;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceData {
    pub id: WorkspaceId,
    pub name: String,
    pub windows: u16,
    pub monitor: String,
}

#[derive(Debug, Clone)]
pub(crate) struct FilterContext<'a> {
    pub show_special: bool,
    pub monitor_specific: bool,
    pub min_workspace_count: usize,
    pub active_workspace_id: WorkspaceId,
    pub bar_monitor: Option<&'a str>,
    pub ignore_patterns: &'a [String],
    pub workspace_monitor_rules: &'a HashMap<WorkspaceId, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct FilteredWorkspace {
    pub id: WorkspaceId,
    pub name: String,
    pub windows: u16,
}

pub(crate) fn filter_workspaces(
    workspaces: &[WorkspaceData],
    ctx: &FilterContext<'_>,
) -> Vec<FilteredWorkspace> {
    let max_id = WorkspaceId::try_from(ctx.min_workspace_count).unwrap_or(WorkspaceId::MAX);

    let mut filtered: Vec<FilteredWorkspace> = workspaces
        .iter()
        .filter(|ws| should_include_workspace(ws.id, ws.windows, &ws.monitor, ctx, max_id))
        .map(|ws| FilteredWorkspace {
            id: ws.id,
            name: ws.name.clone(),
            windows: ws.windows,
        })
        .collect();

    filtered.sort_by_key(|ws| ws.id);

    if ctx.min_workspace_count > 0 {
        add_placeholder_workspaces(&mut filtered, workspaces, ctx, max_id);
    }

    filtered.sort_by_key(|ws| ws.id);
    filtered
}

fn should_include_workspace(
    id: WorkspaceId,
    windows: u16,
    monitor: &str,
    ctx: &FilterContext<'_>,
    max_id: WorkspaceId,
) -> bool {
    if matches_ignore_patterns(id, ctx.ignore_patterns) {
        return false;
    }

    let is_special = id < 0;
    if is_special && !ctx.show_special {
        return false;
    }

    if exceeds_min_count_limit(
        id,
        windows,
        max_id,
        ctx.active_workspace_id,
        ctx.min_workspace_count,
    ) {
        return false;
    }

    if belongs_to_different_monitor(monitor, ctx.monitor_specific, ctx.bar_monitor) {
        return false;
    }

    true
}

fn exceeds_min_count_limit(
    id: WorkspaceId,
    windows: u16,
    max_id: WorkspaceId,
    active_id: WorkspaceId,
    min_count: usize,
) -> bool {
    let has_limit = min_count > 0;
    let is_normal = id > 0;
    let beyond_limit = id > max_id;
    let is_active = id == active_id;
    let is_occupied = windows > 0;

    has_limit && is_normal && beyond_limit && !is_active && !is_occupied
}

fn belongs_to_different_monitor(
    workspace_monitor: &str,
    monitor_specific: bool,
    bar_monitor: Option<&str>,
) -> bool {
    if !monitor_specific {
        return false;
    }
    let Some(bar_mon) = bar_monitor else {
        return false;
    };
    workspace_monitor != bar_mon
}

fn add_placeholder_workspaces(
    filtered: &mut Vec<FilteredWorkspace>,
    all_workspaces: &[WorkspaceData],
    ctx: &FilterContext<'_>,
    max_id: WorkspaceId,
) {
    let mut existing_ids: HashSet<WorkspaceId> = filtered
        .iter()
        .map(|ws| ws.id)
        .chain(
            all_workspaces
                .iter()
                .map(|ws| ws.id)
                .filter(|id| *id > 0 && *id <= max_id),
        )
        .collect();

    for id in 1..=max_id {
        if existing_ids.contains(&id) {
            continue;
        }

        if matches_ignore_patterns(id, ctx.ignore_patterns) {
            continue;
        }

        if ctx.monitor_specific {
            let Some(bar_monitor) = ctx.bar_monitor else {
                continue;
            };
            let rule_monitor = ctx.workspace_monitor_rules.get(&id);
            if rule_monitor.map(String::as_str) != Some(bar_monitor) {
                continue;
            }
        }

        filtered.push(FilteredWorkspace {
            id,
            name: String::new(),
            windows: 0,
        });
        existing_ids.insert(id);
    }

    if ctx.active_workspace_id > 0
        && !existing_ids.contains(&ctx.active_workspace_id)
        && !matches_ignore_patterns(ctx.active_workspace_id, ctx.ignore_patterns)
        && should_include_active_workspace_placeholder(all_workspaces, ctx)
    {
        filtered.push(FilteredWorkspace {
            id: ctx.active_workspace_id,
            name: String::new(),
            windows: 0,
        });
    }
}

fn should_include_active_workspace_placeholder(
    all_workspaces: &[WorkspaceData],
    ctx: &FilterContext<'_>,
) -> bool {
    if !ctx.monitor_specific {
        return true;
    }

    let Some(bar_monitor) = ctx.bar_monitor else {
        return true;
    };

    let active_monitor = all_workspaces
        .iter()
        .find(|ws| ws.id == ctx.active_workspace_id)
        .map(|ws| ws.monitor.as_str())
        .or_else(|| {
            ctx.workspace_monitor_rules
                .get(&ctx.active_workspace_id)
                .map(String::as_str)
        });

    !matches!(active_monitor, Some(monitor) if monitor != bar_monitor)
}

pub(crate) fn monitor_workspaces_sorted(
    bar_monitor: &str,
    workspace_monitor_rules: &HashMap<WorkspaceId, String>,
) -> Vec<WorkspaceId> {
    let mut matching: Vec<_> = workspace_monitor_rules
        .iter()
        .filter(|(_, monitor)| monitor.as_str() == bar_monitor)
        .map(|(id, _)| *id)
        .filter(|id| *id > 0)
        .collect();

    matching.sort_unstable();
    matching
}

pub(crate) fn relative_workspace_number(
    id: WorkspaceId,
    monitor_workspaces: &[WorkspaceId],
) -> WorkspaceId {
    if monitor_workspaces.is_empty() {
        return id;
    }

    monitor_workspaces
        .iter()
        .position(|&ws_id| ws_id == id)
        .map(|pos| (pos + 1) as WorkspaceId)
        .unwrap_or(id)
}

pub(crate) fn calculate_navigation_index(
    current_idx: usize,
    direction: i64,
    total: usize,
) -> usize {
    if direction > 0 {
        (total - 1).min(current_idx + 1)
    } else if current_idx == 0 {
        current_idx
    } else {
        current_idx - 1
    }
}

pub(crate) fn calculate_navigation_index_wrapping(
    current_idx: usize,
    direction: i64,
    total: usize,
) -> usize {
    if direction > 0 {
        (current_idx + 1) % total
    } else if current_idx == 0 {
        total - 1
    } else {
        current_idx - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod filter_workspaces {
        use super::*;

        fn make_workspace(id: WorkspaceId, monitor: &str) -> WorkspaceData {
            WorkspaceData {
                id,
                name: id.to_string(),
                windows: 1,
                monitor: monitor.to_string(),
            }
        }

        fn make_empty_workspace(id: WorkspaceId, monitor: &str) -> WorkspaceData {
            WorkspaceData {
                id,
                name: id.to_string(),
                windows: 0,
                monitor: monitor.to_string(),
            }
        }

        #[test]
        fn filters_by_monitor_when_monitor_specific() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_workspace(2, "DP-2"),
                make_workspace(3, "DP-1"),
            ];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 0,
                active_workspace_id: 1,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert_eq!(ids, vec![1, 3]);
        }

        #[test]
        fn excludes_special_workspaces_by_default() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(-99, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert_eq!(ids, vec![1]);
        }

        #[test]
        fn includes_special_when_enabled() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(-99, "DP-1")];

            let ctx = FilterContext {
                show_special: true,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert_eq!(ids, vec![-99, 1]);
        }

        #[test]
        fn respects_ignore_patterns() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_workspace(2, "DP-1"),
                make_workspace(10, "DP-1"),
            ];

            let patterns = vec!["10".to_string()];
            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert_eq!(ids, vec![1, 2]);
        }

        #[test]
        fn adds_placeholder_workspaces_up_to_min_count() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 3,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert_eq!(ids, vec![1, 2, 3]);
        }

        #[test]
        fn always_includes_active_workspace() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 3,
                active_workspace_id: 5,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(ids.contains(&5));
        }

        #[test]
        fn includes_occupied_workspace_beyond_min_count_limit() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 8,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(ids.contains(&9));
        }

        #[test]
        fn excludes_empty_workspace_beyond_min_count_limit_when_not_active() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_empty_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 8,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_does_not_readd_active_workspace_from_other_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-2")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace_id: 9,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_does_not_readd_active_workspace_from_other_monitor_rule() {
            let workspaces = vec![make_workspace(1, "DP-1")];
            let mut rules = HashMap::new();
            rules.insert(9, "DP-2".to_string());

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace_id: 9,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_includes_occupied_workspace_beyond_min_count_on_bar_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace_id: 1,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(ids.contains(&9));
        }

        #[test]
        fn monitor_specific_excludes_occupied_workspace_beyond_min_count_on_other_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-2")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace_id: 1,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_readds_active_workspace_placeholder_when_rule_matches_bar_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1")];
            let mut rules = HashMap::new();
            rules.insert(9, "DP-1".to_string());

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace_id: 9,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();
            assert!(ids.contains(&9));
        }

        #[test]
        fn monitor_specific_placeholders_require_explicit_rules() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let mut rules = HashMap::new();
            rules.insert(1, "DP-1".to_string());
            rules.insert(3, "DP-1".to_string());

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 5,
                active_workspace_id: 1,
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();

            assert!(ids.contains(&1));
            assert!(ids.contains(&3));
            assert!(!ids.contains(&2));
            assert!(!ids.contains(&4));
            assert!(!ids.contains(&5));
        }

        #[test]
        fn global_mode_adds_all_placeholders() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 5,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();

            assert_eq!(ids, vec![1, 2, 3, 4, 5]);
        }

        #[test]
        fn placeholders_respect_ignore_patterns() {
            let workspaces = vec![make_workspace(1, "DP-1")];
            let patterns = vec!["3".to_string(), "5".to_string()];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 5,
                active_workspace_id: 1,
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();

            assert_eq!(ids, vec![1, 2, 4]);
        }

        #[test]
        fn active_workspace_respects_ignore_patterns() {
            let workspaces = vec![make_workspace(1, "DP-1")];
            let patterns = vec!["10".to_string()];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 5,
                active_workspace_id: 10,
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids: Vec<_> = result.iter().map(|ws| ws.id).collect();

            assert!(!ids.contains(&10));
        }
    }

    mod relative_workspace_number {
        use super::*;

        #[test]
        fn returns_id_when_empty_list() {
            assert_eq!(relative_workspace_number(5, &[]), 5);
        }

        #[test]
        fn returns_position_plus_one() {
            let workspaces = vec![4, 5, 6];
            assert_eq!(relative_workspace_number(5, &workspaces), 2);
        }

        #[test]
        fn returns_id_when_not_found() {
            let workspaces = vec![1, 2, 3];
            assert_eq!(relative_workspace_number(10, &workspaces), 10);
        }
    }

    mod calculate_navigation_index {
        use super::*;

        #[test]
        fn dont_wrap_forward_at_end() {
            assert_eq!(calculate_navigation_index(4, 1, 5), 4);
        }

        #[test]
        fn dont_wrap_backward_at_start() {
            assert_eq!(calculate_navigation_index(0, -1, 5), 0);
        }

        #[test]
        fn moves_forward() {
            assert_eq!(calculate_navigation_index(2, 1, 5), 3);
        }

        #[test]
        fn moves_backward() {
            assert_eq!(calculate_navigation_index(2, -1, 5), 1);
        }
    }

    mod calculate_navigation_index_wrapping {
        use super::*;

        #[test]
        fn wraps_forward_at_end() {
            assert_eq!(calculate_navigation_index_wrapping(4, 1, 5), 0);
        }

        #[test]
        fn wraps_backward_at_start() {
            assert_eq!(calculate_navigation_index_wrapping(0, -1, 5), 4);
        }

        #[test]
        fn moves_forward() {
            assert_eq!(calculate_navigation_index_wrapping(2, 1, 5), 3);
        }

        #[test]
        fn moves_backward() {
            assert_eq!(calculate_navigation_index_wrapping(2, -1, 5), 1);
        }
    }

    mod monitor_workspaces_sorted {
        use super::*;

        #[test]
        fn filters_and_sorts() {
            let mut rules = HashMap::new();
            rules.insert(5, "DP-1".to_string());
            rules.insert(1, "DP-1".to_string());
            rules.insert(3, "DP-2".to_string());
            rules.insert(2, "DP-1".to_string());

            let result = monitor_workspaces_sorted("DP-1", &rules);
            assert_eq!(result, vec![1, 2, 5]);
        }

        #[test]
        fn excludes_negative_ids() {
            let mut rules = HashMap::new();
            rules.insert(1, "DP-1".to_string());
            rules.insert(-99, "DP-1".to_string());

            let result = monitor_workspaces_sorted("DP-1", &rules);
            assert_eq!(result, vec![1]);
        }
    }
}
