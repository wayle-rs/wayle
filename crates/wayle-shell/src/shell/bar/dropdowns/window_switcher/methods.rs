//! [`WindowSwitcherDropdown`] private impl methods: rebuilding the row
//! list, mouse-click activation, and keyboard-cycle navigation.
//!
//! Clicking a row, cycling with Tab, and committing with Return all
//! activate the highlighted window immediately - matching classic
//! alt-tab, where the preview already reflects the real focus and Return
//! just confirms/closes. `cycle_cancel` (Escape) is the only path that
//! does *not* activate; it restores the originally-active window and
//! closes the popover, so backing out of a cycle doesn't leave focus on
//! whatever was last previewed.

use gtk::prelude::*;
use relm4::gtk;

use super::{WindowSwitcherDropdown, messages::WindowInfo, row::WindowRowMsg};
use crate::{glob, process};

/// Best-effort reset of Sway's `"altTab"` binding mode back to `"default"`.
///
/// Sway only exits the mode via the `Return`/`Escape` bindings in
/// `~/.config/sway/config` - clicking a row, or any other way the popover
/// closes, leaves Sway stuck in `"altTab"` with no visual indicator. While
/// stuck, every keybinding *not* defined in that mode (i.e. almost all of
/// them) is silently swallowed and falls through to the focused window as
/// plain text instead of triggering its binding. Calling this from every
/// exit path removes Return/Escape as the single point of failure.
/// No-ops quietly on non-Sway compositors (`SWAYSOCK` unset).
fn reset_sway_mode() {
    if std::env::var_os("SWAYSOCK").is_some() {
        process::run_if_set("swaymsg mode default");
    }
}

impl WindowSwitcherDropdown {
    pub(super) fn rebuild_rows(&mut self) {
        let toplevels = self.service.toplevels.get();
        let config = &self.config.config().modules.window_switcher;
        let ignore_patterns = config.ignore_app_id.get();
        let max_title_length = config.max_title_length.get();

        let mut entries: Vec<_> = toplevels
            .values()
            .filter(|toplevel| {
                let app_id = toplevel.app_id.get();
                !app_id
                    .as_deref()
                    .is_some_and(|id| ignore_patterns.iter().any(|p| glob::matches(p, id)))
            })
            .map(|toplevel| {
                let title = toplevel.title.get().unwrap_or_default();
                WindowInfo {
                    key: toplevel.key,
                    title: truncate(&title, max_title_length),
                    app_id: toplevel.app_id.get().unwrap_or_default(),
                    is_active: toplevel.is_activated(),
                    is_highlighted: false,
                }
            })
            .collect();
        entries.sort_by_key(|info| info.key);

        self.ordered_keys = entries.iter().map(|info| info.key).collect();

        // Keep the highlight on the same window across a rebuild (e.g. a
        // title changing mid-cycle), falling back to none if it closed.
        if let Some(key) = self.highlighted_key() {
            self.highlighted_index = self.ordered_keys.iter().position(|k| *k == key);
        }
        if let Some(index) = self.highlighted_index
            && let Some(info) = entries.get_mut(index)
        {
            info.is_highlighted = true;
        }

        let mut guard = self.rows.guard();
        guard.clear();
        for info in entries {
            guard.push_back(info);
        }
    }

    pub(super) fn activate_row(&mut self, index: usize, popover: &gtk::Popover) {
        self.highlighted_index = None;
        self.cycle_origin_key = None;
        if let Some(key) = self.ordered_keys.get(index).copied() {
            self.service.activate_toplevel(key);
        }
        reset_sway_mode();
        popover.popdown();
    }

    /// Advances the highlighted selection.
    ///
    /// Opening the popover itself is the bar module's job (it owns the
    /// anchor widget the popover must be parented to before `popup()` can
    /// work - this component only has the popover, never a parent for
    /// it). It watches the same IPC property independently and opens the
    /// dropdown through the normal click-dispatch path before, after, or
    /// concurrently with this running; the order doesn't matter since the
    /// two only touch the popover's visibility and this component's
    /// highlight state respectively.
    ///
    /// Only sends a per-row highlight toggle rather than calling
    /// `rebuild_rows` - clearing and re-pushing every row on each step was
    /// disruptive enough to the `ListBox` that the popover's autohide
    /// closed it mid-cycle.
    pub(super) fn cycle_step(&mut self) {
        if self.highlighted_index.is_none() {
            // Start of a new cycle: begin from the active window so the
            // first Tab moves to the next one, matching classic alt-tab.
            // Remember it so `cycle_cancel` can restore it.
            self.highlighted_index = self.ordered_keys.iter().position(|key| {
                self.service
                    .toplevel(*key)
                    .is_some_and(|toplevel| toplevel.is_activated())
            });
            self.cycle_origin_key = self.highlighted_key();
        }

        if self.ordered_keys.is_empty() {
            self.highlighted_index = None;
            return;
        }

        let previous = self.highlighted_index;
        let next = match previous {
            Some(index) => (index + 1) % self.ordered_keys.len(),
            None => 0,
        };
        self.highlighted_index = Some(next);
        self.set_row_highlight(previous, false);
        self.set_row_highlight(Some(next), true);

        // Activate immediately so the preview reflects real focus, like
        // classic alt-tab - not just on commit.
        if let Some(key) = self.ordered_keys.get(next).copied() {
            self.service.activate_toplevel(key);
        }
    }

    /// Activates the highlighted window (Return key) and closes the
    /// popover.
    ///
    /// `popdown()` is safe to call here even in the edge case where a
    /// commit arrives without a preceding step (so the popover was never
    /// parented/shown) - per GTK docs it's a no-op when not popped up,
    /// unlike `popup()` which needs a parent surface to create its own.
    pub(super) fn cycle_commit(&mut self, popover: &gtk::Popover) {
        if let Some(index) = self.highlighted_index.take() {
            self.set_row_highlight(Some(index), false);
            if let Some(key) = self.ordered_keys.get(index).copied() {
                self.service.activate_toplevel(key);
            }
        }
        self.cycle_origin_key = None;
        reset_sway_mode();
        popover.popdown();
    }

    /// Cancels the in-progress cycle (Escape), restoring whichever window
    /// was active before it started, and closes the popover.
    ///
    /// Without this, Escape only exited Sway's `"altTab"` mode - the
    /// popover stayed open with its layer-shell surface stuck in
    /// `KeyboardMode::OnDemand`, and the window already activated by the
    /// last `cycle_step` stayed focused instead of reverting.
    pub(super) fn cycle_cancel(&mut self, popover: &gtk::Popover) {
        if let Some(index) = self.highlighted_index.take() {
            self.set_row_highlight(Some(index), false);
        }
        if let Some(key) = self.cycle_origin_key.take() {
            self.service.activate_toplevel(key);
        }
        reset_sway_mode();
        popover.popdown();
    }

    fn set_row_highlight(&mut self, index: Option<usize>, value: bool) {
        if let Some(index) = index {
            self.rows.send(index, WindowRowMsg::SetHighlighted(value));
        }
    }

    fn highlighted_key(&self) -> Option<u32> {
        self.highlighted_index
            .and_then(|index| self.ordered_keys.get(index).copied())
    }
}

fn truncate(text: &str, max_chars: u32) -> String {
    if max_chars == 0 {
        return text.to_string();
    }
    let max_chars = max_chars as usize;
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_no_limit() {
        assert_eq!(truncate("hello world", 0), "hello world");
    }

    #[test]
    fn truncate_under_limit_unchanged() {
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn truncate_over_limit_adds_ellipsis() {
        assert_eq!(truncate("hello world", 5), "hell…");
    }
}
