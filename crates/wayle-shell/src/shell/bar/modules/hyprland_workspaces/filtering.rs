use std::collections::{HashMap, HashSet};

use wayle_hyprland::{WorkspaceAddress, WorkspaceId, WorkspaceInfo, WorkspaceKind};

use super::helpers::matches_ignore_patterns;

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceData {
    pub id: Option<WorkspaceId>,
    pub address: WorkspaceAddress,
    pub kind: WorkspaceKind,
    pub name: String,
    pub windows: u16,
    pub monitor: String,
}

/// The workspace the bar highlights as active.
///
/// `id` is only set for numbered workspaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveWorkspace {
    pub id: Option<WorkspaceId>,
    pub address: WorkspaceAddress,
}

impl From<WorkspaceInfo> for ActiveWorkspace {
    fn from(info: WorkspaceInfo) -> Self {
        Self {
            id: info.id,
            address: info.address,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FilterContext<'a> {
    pub show_special: bool,
    pub monitor_specific: bool,
    pub min_workspace_count: usize,
    pub active_workspace: Option<ActiveWorkspace>,
    pub bar_monitor: Option<&'a str>,
    pub ignore_patterns: &'a [String],
    pub workspace_monitor_rules: &'a HashMap<WorkspaceId, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct FilteredWorkspace {
    pub id: Option<WorkspaceId>,
    pub address: WorkspaceAddress,
    pub kind: WorkspaceKind,
    pub name: String,
    pub windows: u16,
}

impl FilteredWorkspace {
    fn placeholder(id: WorkspaceId) -> Self {
        Self {
            id: Some(id),
            address: WorkspaceAddress::new(id.to_string()),
            kind: WorkspaceKind::Normal,
            name: String::new(),
            windows: 0,
        }
    }

    /// Workspace selector for Hyprland's dispatchers.
    ///
    /// Numbered workspaces dispatch by number. Named and special ones go through
    /// `name:<name>`, because Hyprland reads a bare negative or non-numeric
    /// selector as something else. Returns `None` for an unnumbered workspace
    /// with no name.
    pub fn dispatch_selector(&self) -> Option<String> {
        if let Some(id) = self.id {
            return Some(id.to_string());
        }

        if self.name.is_empty() {
            return None;
        }

        Some(format!("name:{}", self.name))
    }

    /// Bar order: numbered by number, then named workspaces by name, then
    /// special workspaces by name.
    fn display_order(&self) -> (u8, Option<WorkspaceId>, &str) {
        let group = match (self.id, self.kind) {
            (Some(_), _) => 0,
            (None, WorkspaceKind::Special) => 2,
            (None, _) => 1,
        };

        (group, self.id, &self.name)
    }
}

pub(crate) fn filter_workspaces(
    workspaces: &[WorkspaceData],
    ctx: &FilterContext<'_>,
) -> Vec<FilteredWorkspace> {
    let max_id = WorkspaceId::try_from(ctx.min_workspace_count).unwrap_or(WorkspaceId::MAX);

    let mut filtered: Vec<FilteredWorkspace> = workspaces
        .iter()
        .filter(|ws| should_include_workspace(ws, ctx, max_id))
        .map(|ws| FilteredWorkspace {
            id: ws.id,
            address: ws.address.clone(),
            kind: ws.kind,
            name: ws.name.clone(),
            windows: ws.windows,
        })
        .collect();

    if ctx.min_workspace_count > 0 {
        add_placeholder_workspaces(&mut filtered, workspaces, ctx, max_id);
    }

    filtered.sort_by(|left, right| left.display_order().cmp(&right.display_order()));
    filtered
}

fn should_include_workspace(
    workspace: &WorkspaceData,
    ctx: &FilterContext<'_>,
    max_id: WorkspaceId,
) -> bool {
    if matches_ignore_patterns(&workspace.name, workspace.id, ctx.ignore_patterns) {
        return false;
    }

    let is_special = workspace.kind == WorkspaceKind::Special;
    if is_special && !ctx.show_special {
        return false;
    }

    if exceeds_min_count_limit(workspace, max_id, ctx) {
        return false;
    }

    if belongs_to_different_monitor(&workspace.monitor, ctx.monitor_specific, ctx.bar_monitor) {
        return false;
    }

    true
}

fn exceeds_min_count_limit(
    workspace: &WorkspaceData,
    max_id: WorkspaceId,
    ctx: &FilterContext<'_>,
) -> bool {
    let Some(id) = workspace.id else {
        return false;
    };

    let has_limit = ctx.min_workspace_count > 0;
    let is_normal = id > 0;
    let beyond_limit = id > max_id;
    let is_active = ctx
        .active_workspace
        .as_ref()
        .is_some_and(|active| active.address == workspace.address);
    let is_occupied = workspace.windows > 0;

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
    let existing_numbered = all_workspaces
        .iter()
        .filter_map(|ws| ws.id)
        .filter(|id| *id > 0 && *id <= max_id);

    let mut existing_ids: HashSet<WorkspaceId> = filtered
        .iter()
        .filter_map(|ws| ws.id)
        .chain(existing_numbered)
        .collect();

    for id in 1..=max_id {
        if existing_ids.contains(&id) {
            continue;
        }

        if matches_ignore_patterns("", Some(id), ctx.ignore_patterns) {
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

        filtered.push(FilteredWorkspace::placeholder(id));
        existing_ids.insert(id);
    }

    let Some(active_id) = ctx.active_workspace.as_ref().and_then(|active| active.id) else {
        return;
    };

    if active_id > 0
        && !existing_ids.contains(&active_id)
        && !matches_ignore_patterns("", Some(active_id), ctx.ignore_patterns)
        && should_include_active_workspace_placeholder(all_workspaces, ctx, active_id)
    {
        filtered.push(FilteredWorkspace::placeholder(active_id));
    }
}

fn should_include_active_workspace_placeholder(
    all_workspaces: &[WorkspaceData],
    ctx: &FilterContext<'_>,
    active_id: WorkspaceId,
) -> bool {
    if !ctx.monitor_specific {
        return true;
    }

    let Some(bar_monitor) = ctx.bar_monitor else {
        return true;
    };

    let active_monitor = all_workspaces
        .iter()
        .find(|ws| ws.id == Some(active_id))
        .map(|ws| ws.monitor.as_str())
        .or_else(|| {
            ctx.workspace_monitor_rules
                .get(&active_id)
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
                id: Some(id),
                address: WorkspaceAddress::new(id.to_string()),
                kind: WorkspaceKind::Normal,
                name: id.to_string(),
                windows: 1,
                monitor: monitor.to_string(),
            }
        }

        fn make_empty_workspace(id: WorkspaceId, monitor: &str) -> WorkspaceData {
            WorkspaceData {
                windows: 0,
                ..make_workspace(id, monitor)
            }
        }

        fn make_unnumbered_workspace(name: &str, kind: WorkspaceKind) -> WorkspaceData {
            WorkspaceData {
                id: None,
                address: WorkspaceAddress::new(name),
                kind,
                name: name.to_string(),
                windows: 1,
                monitor: "DP-1".to_string(),
            }
        }

        fn active(id: WorkspaceId) -> ActiveWorkspace {
            ActiveWorkspace {
                id: Some(id),
                address: WorkspaceAddress::new(id.to_string()),
            }
        }

        fn numbered_ids(workspaces: &[FilteredWorkspace]) -> Vec<WorkspaceId> {
            workspaces
                .iter()
                .filter_map(|workspace| workspace.id)
                .collect()
        }

        fn addresses(workspaces: &[FilteredWorkspace]) -> Vec<&str> {
            workspaces
                .iter()
                .map(|workspace| workspace.address.as_str())
                .collect()
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
                active_workspace: Some(active(1)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert_eq!(ids, vec![1, 3]);
        }

        #[test]
        fn excludes_special_workspaces_when_disabled() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_unnumbered_workspace("special:scratch", WorkspaceKind::Special),
            ];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(addresses(&result), vec!["1"]);
        }

        #[test]
        fn includes_special_when_enabled() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_unnumbered_workspace("special:scratch", WorkspaceKind::Special),
            ];

            let ctx = FilterContext {
                show_special: true,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(addresses(&result), vec!["1", "special:scratch"]);
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
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert_eq!(ids, vec![1, 2]);
        }

        #[test]
        fn adds_placeholder_workspaces_up_to_min_count() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 3,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert_eq!(ids, vec![1, 2, 3]);
        }

        #[test]
        fn always_includes_active_workspace() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 3,
                active_workspace: Some(active(5)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert!(ids.contains(&5));
        }

        #[test]
        fn includes_occupied_workspace_beyond_min_count_limit() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 8,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert!(ids.contains(&9));
        }

        #[test]
        fn excludes_empty_workspace_beyond_min_count_limit_when_not_active() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_empty_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 8,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_does_not_readd_active_workspace_from_other_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-2")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace: Some(active(9)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
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
                active_workspace: Some(active(9)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert!(!ids.contains(&9));
        }

        #[test]
        fn monitor_specific_includes_occupied_workspace_beyond_min_count_on_bar_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace: Some(active(1)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
            assert!(ids.contains(&9));
        }

        #[test]
        fn monitor_specific_excludes_occupied_workspace_beyond_min_count_on_other_monitor() {
            let workspaces = vec![make_workspace(1, "DP-1"), make_workspace(9, "DP-2")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: true,
                min_workspace_count: 8,
                active_workspace: Some(active(1)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
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
                active_workspace: Some(active(9)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);
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
                active_workspace: Some(active(1)),
                bar_monitor: Some("DP-1"),
                ignore_patterns: &[],
                workspace_monitor_rules: &rules,
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);

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
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);

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
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);

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
                active_workspace: Some(active(10)),
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            let ids = numbered_ids(&result);

            assert!(!ids.contains(&10));
        }

        #[test]
        fn shows_named_workspaces_when_special_disabled() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_unnumbered_workspace("music", WorkspaceKind::Normal),
            ];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(addresses(&result), vec!["1", "music"]);
        }

        #[test]
        fn orders_numbered_then_named_then_special() {
            let workspaces = vec![
                make_workspace(2, "DP-1"),
                make_unnumbered_workspace("special:scratch", WorkspaceKind::Special),
                make_workspace(1, "DP-1"),
                make_unnumbered_workspace("web", WorkspaceKind::Normal),
                make_unnumbered_workspace("chat", WorkspaceKind::Normal),
            ];

            let ctx = FilterContext {
                show_special: true,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(
                addresses(&result),
                vec!["1", "2", "chat", "web", "special:scratch"]
            );
        }

        #[test]
        fn ignore_patterns_match_named_workspaces_by_name() {
            let workspaces = vec![
                make_workspace(1, "DP-1"),
                make_unnumbered_workspace("music", WorkspaceKind::Normal),
            ];
            let patterns = vec!["mus*".to_string()];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 0,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &patterns,
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(addresses(&result), vec!["1"]);
        }

        #[test]
        fn placeholders_use_the_number_as_address() {
            let workspaces = vec![make_workspace(1, "DP-1")];

            let ctx = FilterContext {
                show_special: false,
                monitor_specific: false,
                min_workspace_count: 2,
                active_workspace: Some(active(1)),
                bar_monitor: None,
                ignore_patterns: &[],
                workspace_monitor_rules: &HashMap::new(),
            };

            let result = filter_workspaces(&workspaces, &ctx);
            assert_eq!(addresses(&result), vec!["1", "2"]);
        }
    }

    mod dispatch_selector {
        use super::*;

        fn workspace(id: Option<WorkspaceId>, name: &str) -> FilteredWorkspace {
            FilteredWorkspace {
                id,
                address: WorkspaceAddress::new(name),
                kind: WorkspaceKind::Normal,
                name: name.to_string(),
                windows: 0,
            }
        }

        #[test]
        fn numbered_workspace_dispatches_by_number() {
            assert_eq!(
                workspace(Some(3), "3").dispatch_selector().as_deref(),
                Some("3")
            );
        }

        #[test]
        fn named_workspace_dispatches_by_name() {
            assert_eq!(
                workspace(None, "music").dispatch_selector().as_deref(),
                Some("name:music")
            );
        }

        #[test]
        fn unnumbered_workspace_without_name_has_no_selector() {
            assert_eq!(workspace(None, "").dispatch_selector(), None);
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
        fn wraps_forward_at_end() {
            assert_eq!(calculate_navigation_index(4, 1, 5), 0);
        }

        #[test]
        fn wraps_backward_at_start() {
            assert_eq!(calculate_navigation_index(0, -1, 5), 4);
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
