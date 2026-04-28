use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::Arc,
};

use glob::Pattern;
use wayle_config::schemas::modules::{DisplayMode, Numbering};
use wayle_hyprland::{Address, Client, WorkspaceId};

use super::filtering::relative_workspace_number;
use crate::shell::bar::icons::{DEFAULT_APP_ICON_MAP, matches_glob};

pub(crate) struct IconContext<'a> {
    pub user_map: &'a HashMap<String, String>,
    pub fallback: &'a str,
}

pub(crate) struct WindowInfo<'a> {
    pub class: &'a str,
    pub title: &'a str,
}

enum MatchTarget {
    Class(String),
    Title(String),
}

fn parse_pattern(pattern: &str) -> MatchTarget {
    if let Some(suffix) = pattern.strip_prefix("title:") {
        MatchTarget::Title(suffix.to_lowercase())
    } else if let Some(suffix) = pattern.strip_prefix("class:") {
        MatchTarget::Class(suffix.to_lowercase())
    } else {
        MatchTarget::Class(pattern.to_lowercase())
    }
}

fn workspace_class_key(class: &str) -> String {
    class.to_lowercase()
}

pub(crate) fn resolve_app_icon(window: &WindowInfo<'_>, ctx: &IconContext<'_>) -> String {
    for (pattern, icon) in ctx.user_map.iter() {
        let matches = match parse_pattern(pattern) {
            MatchTarget::Class(p) => matches_glob(window.class, &p),
            MatchTarget::Title(p) => matches_glob(window.title, &p),
        };
        if matches {
            return icon.clone();
        }
    }

    for (pattern, icon) in DEFAULT_APP_ICON_MAP {
        if matches_glob(window.class, &pattern.to_lowercase()) {
            return (*icon).to_string();
        }
    }

    ctx.fallback.to_string()
}

pub(crate) struct ResolvedIcon {
    pub icon_name: String,
    pub addresses: Vec<Address>,
}

pub(crate) fn resolve_workspace_icons(
    workspace_id: WorkspaceId,
    clients: &[Arc<Client>],
    ctx: &IconContext<'_>,
    dedupe: bool,
) -> Vec<ResolvedIcon> {
    let mut icons: Vec<ResolvedIcon> = Vec::new();
    let mut seen: HashMap<String, usize> = HashMap::new();

    for client in clients
        .iter()
        .filter(|client| client.workspace.get().id == workspace_id)
    {
        let class = client.class.get();
        let title = client.title.get();
        let address = client.address.get();
        let key = workspace_class_key(&class);

        if dedupe {
            if let Some(&idx) = seen.get(&key) {
                icons[idx].addresses.push(address);
                continue;
            }
            seen.insert(key, icons.len());
        }

        let window = WindowInfo {
            class: &class,
            title: &title,
        };
        icons.push(ResolvedIcon {
            icon_name: resolve_app_icon(&window, ctx),
            addresses: vec![address],
        });
    }

    icons
}

pub(crate) fn has_title_patterns(user_map: &HashMap<String, String>) -> bool {
    user_map.keys().any(|k| k.starts_with("title:"))
}

pub(super) fn format_workspace_label(
    display_id: WorkspaceId,
    absolute_id: WorkspaceId,
    name: &str,
    use_name: bool,
) -> String {
    if use_name && !name.is_empty() {
        let is_default_name = name == absolute_id.to_string();
        if is_default_name {
            display_id.to_string()
        } else {
            name.to_string()
        }
    } else {
        display_id.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkspaceState {
    Active,
    Occupied,
    Empty,
}

impl WorkspaceState {
    pub fn css_class(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Occupied => "occupied",
            Self::Empty => "empty",
        }
    }
}

pub(crate) fn workspace_id_css_class(id: WorkspaceId) -> String {
    if id < 0 {
        format!("workspace-id-neg{}", id.unsigned_abs())
    } else {
        format!("workspace-id-{id}")
    }
}

pub(crate) fn workspace_name_css_class(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    format!("workspace-name-{safe}")
}

pub(crate) fn matches_ignore_patterns(id: WorkspaceId, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let id_str = id.to_string();
    for pattern in patterns {
        if let Ok(glob) = Pattern::new(pattern)
            && glob.matches(&id_str)
        {
            return true;
        }
    }
    false
}

pub(crate) fn determine_workspace_state(is_active: bool, windows: u16) -> WorkspaceState {
    if is_active {
        WorkspaceState::Active
    } else if windows > 0 {
        WorkspaceState::Occupied
    } else {
        WorkspaceState::Empty
    }
}

pub(crate) fn compute_static_css_classes(
    id: WorkspaceId,
    indicator_class: &'static str,
    is_vertical: bool,
) -> Vec<&'static str> {
    let mut classes = vec!["workspace", indicator_class];
    if id < 0 {
        classes.push("special");
    }
    if is_vertical {
        classes.push("vertical");
    }
    classes
}

pub(crate) fn collect_button_css_classes<'a>(
    static_classes: &[&'a str],
    css_id_class: &'a str,
    state: WorkspaceState,
    is_urgent: bool,
) -> Vec<&'a str> {
    let mut classes: Vec<&str> = static_classes.to_vec();
    classes.push(css_id_class);
    classes.push(state.css_class());
    if is_urgent {
        classes.push("urgent");
    }
    classes
}

pub(crate) fn should_show_divider(
    show_app_icons: bool,
    divider: &str,
    display_mode: DisplayMode,
) -> bool {
    show_app_icons && !divider.is_empty() && display_mode != DisplayMode::None
}

pub(crate) fn compute_display_id(
    id: WorkspaceId,
    numbering: Numbering,
    bar_monitor: Option<&str>,
    monitor_workspaces: &[WorkspaceId],
) -> WorkspaceId {
    match numbering {
        Numbering::Absolute => id,
        Numbering::Relative => {
            if id <= 0 || bar_monitor.is_none() {
                return id;
            }
            relative_workspace_number(id, monitor_workspaces)
        }
    }
}

pub(crate) fn should_update_for_monitor(
    monitor_specific: bool,
    bar_monitor: Option<&str>,
    event_monitor: &str,
) -> bool {
    if !monitor_specific {
        return true;
    }

    match bar_monitor {
        Some(bar_mon) => bar_mon == event_monitor,
        None => true,
    }
}

pub(crate) fn workspace_contains_urgent_address<A: Eq + Hash>(
    workspace_id: WorkspaceId,
    urgent_addresses: &HashSet<A>,
    client_workspaces: &[(A, WorkspaceId)],
) -> bool {
    if urgent_addresses.is_empty() {
        return false;
    }

    client_workspaces
        .iter()
        .any(|(address, ws_id)| *ws_id == workspace_id && urgent_addresses.contains(address))
}

pub(crate) fn addresses_in_workspace<A: Clone>(
    workspace_id: WorkspaceId,
    client_workspaces: &[(A, WorkspaceId)],
) -> Vec<A> {
    client_workspaces
        .iter()
        .filter(|(_, ws_id)| *ws_id == workspace_id)
        .map(|(addr, _)| addr.clone())
        .collect()
}

pub(crate) fn prune_stale_addresses<A: Eq + std::hash::Hash + Clone>(
    urgent_addresses: &HashSet<A>,
    current_addresses: &HashSet<A>,
) -> HashSet<A> {
    urgent_addresses
        .intersection(current_addresses)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod resolve_app_icon {
        use super::*;

        #[test]
        fn exact_class_match() {
            let user_map = HashMap::new();
            let ctx = IconContext {
                user_map: &user_map,
                fallback: "fallback-icon",
            };
            let window = WindowInfo {
                class: "kitty",
                title: "Terminal",
            };
            assert_eq!(resolve_app_icon(&window, &ctx), "tb-cat-symbolic");
        }

        #[test]
        fn glob_class_match() {
            let user_map = HashMap::new();
            let ctx = IconContext {
                user_map: &user_map,
                fallback: "fallback-icon",
            };
            let window = WindowInfo {
                class: "org.mozilla.firefox",
                title: "Mozilla Firefox",
            };
            assert_eq!(resolve_app_icon(&window, &ctx), "si-firefox-symbolic");
        }

        #[test]
        fn user_override() {
            let mut user_map = HashMap::new();
            user_map.insert("kitty".to_string(), "custom-terminal".to_string());
            let ctx = IconContext {
                user_map: &user_map,
                fallback: "fallback-icon",
            };
            let window = WindowInfo {
                class: "kitty",
                title: "Terminal",
            };
            assert_eq!(resolve_app_icon(&window, &ctx), "custom-terminal");
        }

        #[test]
        fn title_prefix_match() {
            let mut user_map = HashMap::new();
            user_map.insert("title:*YouTube*".to_string(), "ld-youtube".to_string());
            let ctx = IconContext {
                user_map: &user_map,
                fallback: "fallback-icon",
            };
            let window = WindowInfo {
                class: "firefox",
                title: "Watching YouTube - Mozilla Firefox",
            };
            assert_eq!(resolve_app_icon(&window, &ctx), "ld-youtube");
        }

        #[test]
        fn fallback_for_unknown() {
            let user_map = HashMap::new();
            let ctx = IconContext {
                user_map: &user_map,
                fallback: "fallback-icon",
            };
            let window = WindowInfo {
                class: "unknown-app",
                title: "Unknown Window",
            };
            assert_eq!(resolve_app_icon(&window, &ctx), "fallback-icon");
        }
    }

    mod has_title_patterns {
        use super::*;

        #[test]
        fn no_title_patterns() {
            let mut map = HashMap::new();
            map.insert("kitty".to_string(), "icon".to_string());
            map.insert("class:firefox".to_string(), "icon".to_string());
            assert!(!has_title_patterns(&map));
        }

        #[test]
        fn with_title_pattern() {
            let mut map = HashMap::new();
            map.insert("kitty".to_string(), "icon".to_string());
            map.insert("title:*YouTube*".to_string(), "icon".to_string());
            assert!(has_title_patterns(&map));
        }
    }

    mod format_workspace_label {
        use super::*;

        #[test]
        fn use_id_when_name_disabled() {
            assert_eq!(format_workspace_label(1, 1, "main", false), "1");
        }

        #[test]
        fn use_custom_name_when_enabled() {
            assert_eq!(format_workspace_label(1, 1, "main", true), "main");
        }

        #[test]
        fn fallback_to_id_on_empty_name() {
            assert_eq!(format_workspace_label(2, 2, "", true), "2");
        }

        #[test]
        fn negative_id_for_special() {
            assert_eq!(format_workspace_label(-99, -99, "scratchpad", false), "-99");
        }

        #[test]
        fn relative_numbering_with_default_name() {
            assert_eq!(format_workspace_label(1, 4, "4", true), "1");
        }

        #[test]
        fn relative_numbering_with_custom_name() {
            assert_eq!(format_workspace_label(1, 4, "browser", true), "browser");
        }
    }

    mod workspace_class_key {
        use super::*;

        #[test]
        fn normalizes_case_for_dedupe() {
            assert_eq!(workspace_class_key("Kitty"), "kitty");
            assert_eq!(
                workspace_class_key("org.mozilla.Firefox"),
                "org.mozilla.firefox"
            );
        }
    }

    mod determine_workspace_state {
        use super::*;

        #[test]
        fn active_takes_precedence() {
            assert_eq!(determine_workspace_state(true, 5), WorkspaceState::Active);
        }

        #[test]
        fn active_with_no_windows() {
            assert_eq!(determine_workspace_state(true, 0), WorkspaceState::Active);
        }

        #[test]
        fn occupied_when_has_windows() {
            assert_eq!(
                determine_workspace_state(false, 3),
                WorkspaceState::Occupied
            );
        }

        #[test]
        fn empty_when_no_windows() {
            assert_eq!(determine_workspace_state(false, 0), WorkspaceState::Empty);
        }
    }

    mod compute_static_css_classes {
        use super::*;

        #[test]
        fn includes_base_and_indicator() {
            let classes = compute_static_css_classes(1, "indicator-background", false);
            assert!(classes.contains(&"workspace"));
            assert!(classes.contains(&"indicator-background"));
        }

        #[test]
        fn adds_special_for_negative_id() {
            let classes = compute_static_css_classes(-99, "indicator-background", false);
            assert!(classes.contains(&"special"));
        }

        #[test]
        fn no_special_for_positive_id() {
            let classes = compute_static_css_classes(1, "indicator-background", false);
            assert!(!classes.contains(&"special"));
        }

        #[test]
        fn adds_vertical_class() {
            let classes = compute_static_css_classes(1, "indicator-underline", true);
            assert!(classes.contains(&"vertical"));
        }

        #[test]
        fn no_vertical_when_horizontal() {
            let classes = compute_static_css_classes(1, "indicator-underline", false);
            assert!(!classes.contains(&"vertical"));
        }
    }

    mod collect_button_css_classes {
        use super::*;

        #[test]
        fn includes_static_classes() {
            let static_classes = vec!["workspace", "indicator-background"];
            let classes = collect_button_css_classes(
                &static_classes,
                "workspace-id-1",
                WorkspaceState::Active,
                false,
            );
            assert!(classes.contains(&"workspace"));
            assert!(classes.contains(&"indicator-background"));
        }

        #[test]
        fn includes_id_class() {
            let static_classes = vec!["workspace"];
            let classes = collect_button_css_classes(
                &static_classes,
                "workspace-id-5",
                WorkspaceState::Occupied,
                false,
            );
            assert!(classes.contains(&"workspace-id-5"));
        }

        #[test]
        fn includes_state_class() {
            let static_classes = vec!["workspace"];
            let classes = collect_button_css_classes(
                &static_classes,
                "workspace-id-1",
                WorkspaceState::Empty,
                false,
            );
            assert!(classes.contains(&"empty"));
        }

        #[test]
        fn includes_urgent_when_set() {
            let static_classes = vec!["workspace"];
            let classes = collect_button_css_classes(
                &static_classes,
                "workspace-id-1",
                WorkspaceState::Occupied,
                true,
            );
            assert!(classes.contains(&"urgent"));
        }

        #[test]
        fn no_urgent_when_not_set() {
            let static_classes = vec!["workspace"];
            let classes = collect_button_css_classes(
                &static_classes,
                "workspace-id-1",
                WorkspaceState::Active,
                false,
            );
            assert!(!classes.contains(&"urgent"));
        }
    }

    mod should_show_divider {
        use super::*;

        #[test]
        fn true_when_all_conditions_met() {
            assert!(should_show_divider(true, "|", DisplayMode::Label));
        }

        #[test]
        fn false_when_app_icons_disabled() {
            assert!(!should_show_divider(false, "|", DisplayMode::Label));
        }

        #[test]
        fn false_when_divider_empty() {
            assert!(!should_show_divider(true, "", DisplayMode::Label));
        }

        #[test]
        fn false_when_display_mode_none() {
            assert!(!should_show_divider(true, "|", DisplayMode::None));
        }

        #[test]
        fn true_with_icon_mode() {
            assert!(should_show_divider(true, "·", DisplayMode::Icon));
        }
    }

    mod compute_display_id {
        use super::*;

        #[test]
        fn absolute_returns_id() {
            assert_eq!(
                compute_display_id(5, Numbering::Absolute, Some("DP-1"), &[1, 2, 5]),
                5
            );
        }

        #[test]
        fn absolute_returns_id_without_monitor() {
            assert_eq!(compute_display_id(5, Numbering::Absolute, None, &[]), 5);
        }

        #[test]
        fn relative_returns_position() {
            assert_eq!(
                compute_display_id(5, Numbering::Relative, Some("DP-1"), &[4, 5, 6]),
                2
            );
        }

        #[test]
        fn relative_returns_id_for_special_workspace() {
            assert_eq!(
                compute_display_id(-99, Numbering::Relative, Some("DP-1"), &[1, 2, 3]),
                -99
            );
        }

        #[test]
        fn relative_returns_id_without_monitor() {
            assert_eq!(
                compute_display_id(5, Numbering::Relative, None, &[1, 2, 5]),
                5
            );
        }

        #[test]
        fn relative_returns_id_when_not_in_list() {
            assert_eq!(
                compute_display_id(10, Numbering::Relative, Some("DP-1"), &[1, 2, 3]),
                10
            );
        }
    }

    mod should_update_for_monitor {
        use super::*;

        #[test]
        fn always_true_when_not_monitor_specific() {
            assert!(should_update_for_monitor(false, Some("DP-1"), "DP-2"));
        }

        #[test]
        fn true_when_monitors_match() {
            assert!(should_update_for_monitor(true, Some("DP-1"), "DP-1"));
        }

        #[test]
        fn false_when_monitors_differ() {
            assert!(!should_update_for_monitor(true, Some("DP-1"), "DP-2"));
        }

        #[test]
        fn true_when_bar_monitor_none() {
            assert!(should_update_for_monitor(true, None, "DP-2"));
        }
    }

    mod workspace_contains_urgent_address {
        use super::*;

        #[test]
        fn returns_false_for_empty_urgent() {
            let urgent: HashSet<u32> = HashSet::new();
            let clients = vec![(1u32, 1i64), (2, 1), (3, 2)];
            assert!(!workspace_contains_urgent_address(1, &urgent, &clients));
        }

        #[test]
        fn returns_true_when_urgent_in_workspace() {
            let mut urgent = HashSet::new();
            urgent.insert(2u32);
            let clients = vec![(1u32, 1i64), (2, 1), (3, 2)];
            assert!(workspace_contains_urgent_address(1, &urgent, &clients));
        }

        #[test]
        fn returns_false_when_urgent_in_different_workspace() {
            let mut urgent = HashSet::new();
            urgent.insert(3u32);
            let clients = vec![(1u32, 1i64), (2, 1), (3, 2)];
            assert!(!workspace_contains_urgent_address(1, &urgent, &clients));
        }

        #[test]
        fn returns_false_when_address_not_in_clients() {
            let mut urgent = HashSet::new();
            urgent.insert(99u32);
            let clients = vec![(1u32, 1i64), (2, 1), (3, 2)];
            assert!(!workspace_contains_urgent_address(1, &urgent, &clients));
        }
    }

    mod addresses_in_workspace {
        use super::*;

        #[test]
        fn returns_matching_addresses() {
            let clients = vec![(1u32, 1i64), (2, 1), (3, 2), (4, 1)];
            let result = addresses_in_workspace(1, &clients);
            assert_eq!(result, vec![1, 2, 4]);
        }

        #[test]
        fn returns_empty_for_no_matches() {
            let clients = vec![(1u32, 2i64), (2, 2)];
            let result = addresses_in_workspace(1, &clients);
            assert!(result.is_empty());
        }

        #[test]
        fn returns_empty_for_empty_clients() {
            let clients: Vec<(u32, i64)> = vec![];
            let result = addresses_in_workspace(1, &clients);
            assert!(result.is_empty());
        }
    }

    mod prune_stale_addresses {
        use super::*;

        #[test]
        fn keeps_addresses_in_current() {
            let mut urgent = HashSet::new();
            urgent.insert(1u32);
            urgent.insert(2);
            urgent.insert(3);

            let mut current = HashSet::new();
            current.insert(2u32);
            current.insert(3);
            current.insert(4);

            let result = prune_stale_addresses(&urgent, &current);
            assert!(result.contains(&2));
            assert!(result.contains(&3));
            assert!(!result.contains(&1));
            assert_eq!(result.len(), 2);
        }

        #[test]
        fn returns_empty_when_no_overlap() {
            let mut urgent = HashSet::new();
            urgent.insert(1u32);
            urgent.insert(2);

            let mut current = HashSet::new();
            current.insert(3u32);
            current.insert(4);

            let result = prune_stale_addresses(&urgent, &current);
            assert!(result.is_empty());
        }

        #[test]
        fn returns_empty_for_empty_urgent() {
            let urgent: HashSet<u32> = HashSet::new();
            let mut current = HashSet::new();
            current.insert(1u32);

            let result = prune_stale_addresses(&urgent, &current);
            assert!(result.is_empty());
        }
    }

    mod matches_ignore_patterns_tests {
        use super::*;

        #[test]
        fn exact_match() {
            let patterns = vec!["10".to_string()];
            assert!(matches_ignore_patterns(10, &patterns));
            assert!(!matches_ignore_patterns(1, &patterns));
            assert!(!matches_ignore_patterns(100, &patterns));
        }

        #[test]
        fn wildcard_single_char() {
            let patterns = vec!["1?".to_string()];
            assert!(matches_ignore_patterns(10, &patterns));
            assert!(matches_ignore_patterns(11, &patterns));
            assert!(matches_ignore_patterns(19, &patterns));
            assert!(!matches_ignore_patterns(1, &patterns));
            assert!(!matches_ignore_patterns(100, &patterns));
        }

        #[test]
        fn wildcard_multi_char() {
            let patterns = vec!["1*".to_string()];
            assert!(matches_ignore_patterns(1, &patterns));
            assert!(matches_ignore_patterns(10, &patterns));
            assert!(matches_ignore_patterns(100, &patterns));
            assert!(!matches_ignore_patterns(2, &patterns));
        }

        #[test]
        fn empty_patterns() {
            let patterns: Vec<String> = vec![];
            assert!(!matches_ignore_patterns(10, &patterns));
        }
    }
}
