//! Filter and order the workspace list for display.

use super::helpers;

/// Plain-data view of one workspace, used as input to [`collect_displayed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceSnapshot {
    pub key: u32,
    pub name: Option<String>,
    pub coordinates: Vec<u32>,
    pub is_active: bool,
    pub is_urgent: bool,
    pub is_hidden: bool,
}

/// Inputs that govern filtering and ordering.
#[derive(Debug, Clone)]
pub(super) struct FilterContext<'a> {
    pub show_hidden: bool,
    pub ignore_patterns: &'a [String],
}

pub(super) fn collect_displayed(
    snapshots: Vec<WorkspaceSnapshot>,
    ctx: &FilterContext<'_>,
) -> Vec<WorkspaceSnapshot> {
    let mut displayed: Vec<WorkspaceSnapshot> = snapshots
        .into_iter()
        .filter(|snapshot| ctx.show_hidden || !snapshot.is_hidden || snapshot.is_active)
        .filter(|snapshot| {
            !helpers::is_ignored(snapshot.name.as_deref(), snapshot.key, ctx.ignore_patterns)
        })
        .collect();

    displayed.sort_by(|a, b| (&a.coordinates, a.key).cmp(&(&b.coordinates, b.key)));
    displayed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace(key: u32, coordinates: &[u32]) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            key,
            name: None,
            coordinates: coordinates.to_vec(),
            is_active: false,
            is_urgent: false,
            is_hidden: false,
        }
    }

    fn ctx_default<'a>() -> FilterContext<'a> {
        FilterContext {
            show_hidden: true,
            ignore_patterns: &[],
        }
    }

    fn keys(displayed: &[WorkspaceSnapshot]) -> Vec<u32> {
        displayed.iter().map(|snapshot| snapshot.key).collect()
    }

    #[test]
    fn orders_by_coordinates_then_key() {
        let snapshots = vec![
            workspace(3, &[1, 0]),
            workspace(1, &[0, 0]),
            workspace(2, &[0, 1]),
        ];
        let displayed = collect_displayed(snapshots, &ctx_default());
        assert_eq!(keys(&displayed), vec![1, 2, 3]);
    }

    #[test]
    fn falls_back_to_key_order_without_coordinates() {
        let snapshots = vec![workspace(3, &[]), workspace(1, &[]), workspace(2, &[])];
        let displayed = collect_displayed(snapshots, &ctx_default());
        assert_eq!(keys(&displayed), vec![1, 2, 3]);
    }

    #[test]
    fn hides_hidden_workspaces_when_show_hidden_is_false() {
        let mut hidden = workspace(1, &[]);
        hidden.is_hidden = true;
        let snapshots = vec![hidden, workspace(2, &[])];
        let ctx = FilterContext {
            show_hidden: false,
            ignore_patterns: &[],
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(keys(&displayed), vec![2]);
    }

    #[test]
    fn always_shows_active_even_when_hidden() {
        let mut hidden_active = workspace(1, &[]);
        hidden_active.is_hidden = true;
        hidden_active.is_active = true;
        let snapshots = vec![hidden_active];
        let ctx = FilterContext {
            show_hidden: false,
            ignore_patterns: &[],
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(keys(&displayed), vec![1]);
    }

    #[test]
    fn respects_ignore_patterns() {
        let snapshots = vec![workspace(1, &[]), workspace(2, &[]), workspace(10, &[])];
        let ctx = FilterContext {
            show_hidden: true,
            ignore_patterns: &[String::from("10")],
        };
        let displayed = collect_displayed(snapshots, &ctx);
        assert_eq!(keys(&displayed), vec![1, 2]);
    }
}
