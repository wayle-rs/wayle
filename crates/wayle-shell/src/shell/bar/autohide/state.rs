//! Pure state machine for bar autohide behavior.
//!
//! Tracks cursor position, timer state, and popover open/closed events to determine
//! whether the bar should be revealed or hidden.

use std::time::{Duration, Instant};

/// How long after the bar's own surface unmaps to distrust a `HoverEnter`/
/// `HoverMotion` on it.
///
/// Unmapping the bar and mapping the `HoverTrigger` happen back to back in
/// the same step, at the same screen edge. If the pointer is resting near
/// that edge at the moment this happens, the compositor can -- most likely,
/// per live reproduction, though the exact protocol-level mechanism is not
/// confirmed -- still deliver one further motion event to the bar's own
/// (just-unmapped) surface: observed live as `Hide` immediately followed by
/// a spurious `Reveal` a couple of milliseconds later, with no genuine
/// `HoverTrigger` event in between. The trigger's own reactions
/// (`on_trigger_enter`) are exempt: that path exists precisely to catch a
/// real re-approach right after hiding, and gating it here would defeat its
/// purpose.
///
/// Every path that can reveal the bar *while the echo window might still be
/// open* clears `hidden_at` back to `None` (`on_trigger_enter`,
/// `on_popover_open`, `set_enabled(false)`) -- a genuine reveal from one of
/// those means the window is over, whatever caused it. `on_hover_enter`/
/// `on_hover_motion` don't need the same treatment: they're the only things
/// gated by this window in the first place, so by the time either one
/// returns `Reveal` the window has already elapsed. Without the clearing
/// above, a real hover landing inside the window right after a trigger- or
/// popover-driven reveal would be silently swallowed (no `cursor_inside`,
/// no cancelled timer), and the bar would hide out from under a pointer
/// that never left.
const HIDE_ECHO_GRACE: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AutohideAction {
    Reveal,
    Hide,
    ScheduleTimer { duration_ms: u32, token: u64 },
    CancelTimer,
    None,
}

#[derive(Debug)]
pub(crate) struct AutohideState {
    enabled: bool,
    timeout_ms: u32,
    is_revealed: bool,
    cursor_inside: bool,
    current_token: u64,
    hidden_at: Option<Instant>,
}

impl AutohideState {
    pub(crate) fn new(enabled: bool, timeout_ms: u32) -> Self {
        Self {
            enabled,
            timeout_ms,
            is_revealed: true,
            cursor_inside: false,
            current_token: 0,
            hidden_at: None,
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn is_revealed(&self) -> bool {
        if !self.enabled {
            true
        } else {
            self.is_revealed
        }
    }

    pub(crate) fn is_cursor_inside(&self) -> bool {
        self.cursor_inside
    }

    /// Returns true if `token` matches the currently active timer
    /// generation.
    ///
    /// Used by `on_autohide_timeout` to tell apart "this expiry was already
    /// superseded by a newer schedule" (a stale token; some other event
    /// already re-armed things correctly) from "this expiry is still
    /// current but blocked" (e.g. a popover the state machine doesn't own a
    /// close event for) -- the two need different follow-up, and
    /// `on_timeout`'s return value alone can't distinguish them.
    fn is_current_token(&self, token: u64) -> bool {
        token == self.current_token
    }

    pub(crate) fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> AutohideAction {
        self.enabled = enabled;
        if !enabled {
            self.current_token += 1;
            self.is_revealed = true;
            self.hidden_at = None;
            AutohideAction::Reveal
        } else if !self.cursor_inside {
            self.next_timer(false)
        } else {
            AutohideAction::None
        }
    }

    pub(crate) fn on_hover_enter(&mut self) -> AutohideAction {
        if self.is_echo_of_recent_hide() {
            return AutohideAction::None;
        }

        self.cursor_inside = true;
        self.current_token += 1;

        if !self.is_revealed {
            self.is_revealed = true;
            AutohideAction::Reveal
        } else {
            AutohideAction::CancelTimer
        }
    }

    pub(crate) fn on_hover_motion(&mut self) -> AutohideAction {
        if self.is_echo_of_recent_hide() {
            return AutohideAction::None;
        }

        self.cursor_inside = true;
        self.current_token += 1;
        if !self.is_revealed {
            self.is_revealed = true;
            AutohideAction::Reveal
        } else {
            AutohideAction::CancelTimer
        }
    }

    /// True while a `HoverEnter`/`HoverMotion` on the bar's own surface is
    /// still within [`HIDE_ECHO_GRACE`] of the last real hide -- see that
    /// constant's doc comment for why this exists.
    fn is_echo_of_recent_hide(&self) -> bool {
        self.hidden_at
            .is_some_and(|hidden_at| hidden_at.elapsed() < HIDE_ECHO_GRACE)
    }

    pub(crate) fn on_hover_leave(&mut self, popovers_open: bool) -> AutohideAction {
        // While a popover is open, a `Leave` on the bar's own surface is not
        // trustworthy: the popup's own grab makes GTK deliver one even
        // though the pointer never actually left the bar (only observed
        // when a dropdown opens/closes, not a genuine departure). Clearing
        // `cursor_inside` here would desync it from reality -- a later
        // `on_popover_closed` would then wrongly believe the pointer is
        // gone and arm a hide timer under a pointer that's still resting on
        // the bar. Leaving it untouched keeps it accurate for whichever
        // state it was already in.
        if !popovers_open {
            self.cursor_inside = false;
        }
        if !self.enabled {
            return AutohideAction::None;
        }

        if popovers_open {
            self.current_token += 1;
            AutohideAction::CancelTimer
        } else {
            self.next_timer(popovers_open)
        }
    }

    pub(crate) fn on_timeout(&mut self, token: u64, popovers_open: bool) -> AutohideAction {
        if !self.enabled || token != self.current_token {
            return AutohideAction::None;
        }

        if self.cursor_inside || popovers_open {
            return AutohideAction::None;
        }

        // Only a genuine revealed -> hidden transition starts the echo
        // window. Without this guard, a token that times out again while
        // already hidden (reachable via the echo's own paired `HoverLeave`,
        // which is not itself gated) would re-stamp `hidden_at` at a moment
        // with no real map/unmap event attached, reopening the window for
        // no reason.
        if self.is_revealed {
            self.hidden_at = Some(Instant::now());
        }
        self.is_revealed = false;
        AutohideAction::Hide
    }

    /// Full decision for a `BarCmd::AutohideTimeout` firing -- the one entry
    /// point `bar/mod.rs`'s handler should call, rather than reimplementing
    /// this logic inline.
    ///
    /// `registry_open` is `DropdownRegistry::any_open()` -- dropdowns
    /// registered there are fully event-driven; `DropdownClosed` already
    /// re-arms via `on_popover_closed` once one closes, so blocking on one
    /// needs no extra help here. `walk_open` is a popover found only by the
    /// generic tree walk (e.g. a systray context menu), which has no close
    /// event of its own.
    ///
    /// `on_timeout` alone can't tell "this expiry was already superseded by
    /// a newer schedule" apart from "this expiry is still current but
    /// blocked" -- both return `None`. This method draws that distinction
    /// via `is_current_token`: only when the timeout is still current *and*
    /// the only thing blocking it is a walk-only popover does it arm another
    /// poll (reusing `arm_hide_timer`'s `ScheduleTimer` path -- safe here
    /// because every transition that sets `cursor_inside = true` also bumps
    /// the token, so a still-current token guarantees `cursor_inside` is
    /// `false`). A registry-blocked or genuinely-stale timeout is left
    /// alone, exactly as `on_timeout` already handles it.
    pub(crate) fn on_autohide_timeout(
        &mut self,
        token: u64,
        registry_open: bool,
        walk_open: bool,
    ) -> AutohideAction {
        let action = self.on_timeout(token, registry_open || walk_open);

        if action == AutohideAction::None && walk_open && self.is_current_token(token) {
            self.arm_hide_timer()
        } else {
            action
        }
    }

    /// Handles the pointer entering the edge-strip `HoverTrigger` surface --
    /// distinct from the bar's own surface (see `on_hover_enter`).
    ///
    /// Reveals the bar if hidden, but deliberately does NOT set
    /// `cursor_inside`: the pointer is in the trigger strip, not the bar's
    /// own hitbox, and the trigger unmaps as soon as the bar reveals, so no
    /// `HoverLeave` is guaranteed to ever arrive for it. Instead this arms a
    /// hide timer immediately, exactly as `on_hover_leave` would -- giving
    /// the user `timeout_ms` to actually move onto the bar's own surface
    /// (which legitimately cancels this timer via `on_hover_enter`/
    /// `on_hover_motion`) before the bar re-hides. Without this, a pointer
    /// that enters the trigger and then wanders off without ever reaching
    /// the bar itself would leave the bar revealed forever with no path
    /// back to hidden.
    pub(crate) fn on_trigger_enter(&mut self) -> AutohideAction {
        if !self.enabled {
            return AutohideAction::None;
        }
        // A trigger event is proof of a live, genuine pointer interaction --
        // the echo window (if one was even pending) is over. Without this,
        // a real `HoverEnter`/`HoverMotion` landing on the bar within
        // HIDE_ECHO_GRACE of this reveal would be silently swallowed by
        // `is_echo_of_recent_hide`, and the bar would hide again out from
        // under a pointer that never left (see
        // `test_trigger_enter_then_real_hover_within_grace_is_not_swallowed`).
        self.hidden_at = None;
        self.is_revealed = true;
        self.next_timer(false)
    }

    pub(crate) fn on_popover_open(&mut self) -> AutohideAction {
        self.current_token += 1;
        if !self.is_revealed {
            self.is_revealed = true;
            self.hidden_at = None;
            AutohideAction::Reveal
        } else {
            AutohideAction::CancelTimer
        }
    }

    pub(crate) fn on_popover_closed(&mut self) -> AutohideAction {
        self.arm_hide_timer()
    }

    /// Arms a fresh inactivity countdown (or cancels, if the cursor is
    /// already back inside the bar) -- the same "start counting down again"
    /// step needed both when a registry dropdown genuinely just closed
    /// (`on_popover_closed`) and when `on_autohide_timeout` needs to poll
    /// again because a walk-only popover is still open (nothing actually
    /// closed there; this just re-checks after another full timeout).
    fn arm_hide_timer(&mut self) -> AutohideAction {
        if !self.enabled {
            return AutohideAction::None;
        }
        if !self.cursor_inside {
            self.next_timer(false)
        } else {
            AutohideAction::CancelTimer
        }
    }

    fn next_timer(&mut self, popovers_open: bool) -> AutohideAction {
        if popovers_open || self.cursor_inside {
            return AutohideAction::CancelTimer;
        }
        self.current_token += 1;
        AutohideAction::ScheduleTimer {
            duration_ms: self.timeout_ms,
            token: self.current_token,
        }
    }

    /// Test-only seam: pushes `hidden_at` back by [`HIDE_ECHO_GRACE`] (plus a
    /// hair) so a test can simulate "the bar has been hidden for a while"
    /// without an actual sleep. Named for intent rather than poking the
    /// field directly at each call site.
    #[cfg(test)]
    fn backdate_hidden_at_past_grace(&mut self) {
        self.hidden_at = self.hidden_at.map(|t| {
            t.checked_sub(HIDE_ECHO_GRACE + Duration::from_millis(1))
                .unwrap_or(t)
        });
    }
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_autohide_disabled_always_revealed() {
        let mut state = AutohideState::new(false, 3000);
        assert!(state.is_revealed());
        assert_eq!(state.on_hover_leave(false), AutohideAction::None);
        assert_eq!(state.on_timeout(1, false), AutohideAction::None);
    }

    #[test]
    fn test_autohide_enabled_flow() {
        let mut state = AutohideState::new(true, 3000);
        assert!(state.is_revealed());

        let action = state.on_hover_leave(false);
        let token = match action {
            AutohideAction::ScheduleTimer { duration_ms, token } => {
                assert_eq!(duration_ms, 3000);
                token
            }
            _ => panic!("Expected ScheduleTimer"),
        };

        let action = state.on_timeout(token, false);
        assert_eq!(action, AutohideAction::Hide);
        assert!(!state.is_revealed());

        // A real re-approach happens well after HIDE_ECHO_GRACE elapses;
        // backdate to simulate that without an actual sleep (see
        // `test_hover_enter_echo_within_grace_is_ignored` for the near-instant
        // case this constant exists to guard against).
        state.backdate_hidden_at_past_grace();

        let action = state.on_hover_enter();
        assert_eq!(action, AutohideAction::Reveal);
        assert!(state.is_revealed());
    }

    #[test]
    fn test_hover_enter_echo_within_grace_is_ignored() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);

        // No real time has passed (this is a synchronous unit test) -- this
        // stands in for the live-observed case where the compositor
        // delivers one more motion event to the bar's just-unmapped surface
        // a couple of milliseconds after it hides.
        assert_eq!(state.on_hover_enter(), AutohideAction::None);
        assert!(!state.is_revealed());
        assert!(!state.is_cursor_inside());
    }

    #[test]
    fn test_hover_motion_echo_within_grace_is_ignored() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);

        assert_eq!(state.on_hover_motion(), AutohideAction::None);
        assert!(!state.is_revealed());
        assert!(!state.is_cursor_inside());
    }

    #[test]
    fn test_hover_enter_after_grace_period_reveals() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);

        state.backdate_hidden_at_past_grace();

        assert_eq!(state.on_hover_enter(), AutohideAction::Reveal);
        assert!(state.is_revealed());
        assert!(state.is_cursor_inside());
    }

    /// The regression this branch's live testing actually hit: a
    /// trigger-driven reveal clears the echo window immediately (see
    /// `on_trigger_enter`'s doc comment), so a genuine hover landing on the
    /// bar's own surface right after -- still well inside HIDE_ECHO_GRACE --
    /// must NOT be swallowed. Before that fix, this sequence silently
    /// dropped the real hover (`cursor_inside` stayed `false`) and the bar
    /// would hide again out from under a pointer that never left.
    ///
    /// Deliberately does NOT go through `hidden_state()`: that helper
    /// backdates `hidden_at` past the grace window on its own, which would
    /// make this test pass identically whether or not `on_trigger_enter`
    /// actually clears it -- exactly the gap a prior review caught. The
    /// hide here is driven inline so `hidden_at` stays genuinely fresh.
    #[test]
    fn test_trigger_enter_then_real_hover_within_grace_is_not_swallowed() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);

        match state.on_trigger_enter() {
            AutohideAction::ScheduleTimer { .. } => {}
            other => panic!("expected ScheduleTimer, got {other:?}"),
        }
        assert!(state.is_revealed());

        // No backdating here -- this hover lands well within HIDE_ECHO_GRACE
        // of the trigger-driven reveal, which is exactly the case that must
        // not be treated as an echo of a hide that didn't happen.
        assert_eq!(state.on_hover_enter(), AutohideAction::CancelTimer);
        assert!(state.is_cursor_inside());
    }

    /// Same shape as the trigger-enter case above, for `on_popover_open`:
    /// a dropdown opening while the bar is hidden reveals it and must clear
    /// `hidden_at` too, or a genuine hover landing right after gets
    /// swallowed the same way. Hide is driven inline, not via
    /// `hidden_state()`, so `hidden_at` stays fresh and the test actually
    /// discriminates.
    #[test]
    fn test_popover_open_then_real_hover_within_grace_is_not_swallowed() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);

        assert_eq!(state.on_popover_open(), AutohideAction::Reveal);
        assert!(state.is_revealed());

        assert_eq!(state.on_hover_enter(), AutohideAction::CancelTimer);
        assert!(state.is_cursor_inside());
    }

    /// Proves `on_timeout` only re-stamps `hidden_at` on a genuine
    /// `revealed -> hidden` transition, not on a phantom timeout that fires
    /// while already hidden (reachable via the echo's own paired
    /// `HoverLeave`, which isn't itself gated). Deliberately backdates the
    /// *first* hide's `hidden_at` past the grace window before triggering
    /// the phantom second timeout: if `on_timeout` incorrectly re-stamped
    /// it, the following `on_hover_enter` would find a fresh timestamp and
    /// get swallowed; since it doesn't, the stale timestamp from the first
    /// hide stands and the hover goes through.
    #[test]
    fn test_phantom_timeout_while_already_hidden_does_not_restamp_hidden_at() {
        let mut state = AutohideState::new(true, 3000);
        let first_token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(first_token, false), AutohideAction::Hide);
        state.backdate_hidden_at_past_grace();

        // A stray echo of the bar's own HoverLeave, not gated by
        // HIDE_ECHO_GRACE, arrives while the bar is already hidden and arms
        // a second timer.
        let second_token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };

        // That phantom timer fires. If this re-stamped `hidden_at`, the
        // hover below would be swallowed.
        assert_eq!(state.on_timeout(second_token, false), AutohideAction::Hide);

        assert_eq!(state.on_hover_enter(), AutohideAction::Reveal);
        assert!(state.is_revealed());
        assert!(state.is_cursor_inside());
    }

    #[test]
    fn test_autohide_popover_open_prevents_hide() {
        let mut state = AutohideState::new(true, 3000);
        let action = state.on_hover_leave(true);
        assert_eq!(action, AutohideAction::CancelTimer);

        let action = state.on_timeout(100, true);
        assert_eq!(action, AutohideAction::None);
        assert!(state.is_revealed());
    }

    /// Reproduces the live bug: pointer rests on the bar, a dropdown opens
    /// (its popup grab makes GTK deliver a spurious `Leave` to the bar's own
    /// surface -- `on_hover_leave(true)`), then the dropdown closes (e.g. a
    /// click elsewhere on the bar dismisses it via GTK's native popover
    /// autohide) while the pointer never actually left the bar. Because
    /// `on_hover_leave` unconditionally clears `cursor_inside`, the
    /// subsequent `on_popover_closed` wrongly believes the pointer is gone
    /// and arms a hide timer -- which fires and hides the bar out from under
    /// a pointer resting right on top of it.
    #[test]
    fn test_popover_close_after_spurious_leave_does_not_arm_hide_timer() {
        let mut state = AutohideState::new(true, 100);

        // Pointer genuinely rests on the bar before the dropdown opens.
        assert_eq!(state.on_hover_enter(), AutohideAction::CancelTimer);
        assert!(state.is_cursor_inside());

        assert_eq!(state.on_hover_leave(true), AutohideAction::CancelTimer);

        match state.on_popover_closed() {
            AutohideAction::CancelTimer => {}
            AutohideAction::ScheduleTimer { token, .. } => {
                assert_eq!(
                    state.on_timeout(token, false),
                    AutohideAction::None,
                    "bar hid even though the pointer never left its surface"
                );
            }
            other => panic!("unexpected action: {other:?}"),
        }
        assert!(state.is_revealed());
    }

    #[test]
    fn test_autohide_stale_token_ignored() {
        let mut state = AutohideState::new(true, 3000);
        let action = state.on_hover_leave(false);
        let token1 = match action {
            AutohideAction::ScheduleTimer { token, .. } => token,
            _ => panic!(),
        };

        state.on_hover_enter();
        let action = state.on_hover_leave(false);
        let token2 = match action {
            AutohideAction::ScheduleTimer { token, .. } => token,
            _ => panic!(),
        };
        assert_ne!(token1, token2);

        assert_eq!(state.on_timeout(token1, false), AutohideAction::None);
        assert!(state.is_revealed());

        assert_eq!(state.on_timeout(token2, false), AutohideAction::Hide);
        assert!(!state.is_revealed());
    }

    #[test]
    fn test_is_current_token() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert!(state.is_current_token(token));

        state.on_hover_enter();
        assert!(!state.is_current_token(token));
    }

    /// Exercises the real `on_autohide_timeout` decision method (the one
    /// `BarCmd::AutohideTimeout`'s handler in `bar/mod.rs` actually calls)
    /// for a popover that only the generic tree walk can see (e.g. a
    /// systray context menu, which emits no `DropdownClosed` and so has no
    /// event to re-arm the hide timer on its own).
    ///
    /// Sequence: leave the bar (nothing tracked by the registry is open, so
    /// this arms a normal hide timer) -> first timeout fires while the
    /// walk-only popover is still open (must poll again, not hide) -> a
    /// stale timeout arriving late must stay inert (no double-schedule) ->
    /// a *second* consecutive poll tick fires while the popover is still
    /// open (must poll again, not give up after one cycle) -> a third
    /// timeout fires after the popover has actually closed, which finally
    /// hides the bar.
    #[test]
    fn test_walk_only_popover_polls_back_instead_of_stalling_forever() {
        let mut state = AutohideState::new(true, 3000);

        let first_token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { duration_ms, token } => {
                assert_eq!(duration_ms, 3000);
                token
            }
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };

        // First timeout: walk-only popover (e.g. the systray menu) is still
        // open. Must not hide, and must arm another poll -- this is the gap
        // `on_timeout` alone can't cover.
        let second_token = match state.on_autohide_timeout(first_token, false, true) {
            AutohideAction::ScheduleTimer { duration_ms, token } => {
                assert_eq!(duration_ms, 3000);
                token
            }
            other => panic!("expected ScheduleTimer (poll-back), got {other:?}"),
        };
        assert!(state.is_revealed());
        assert_ne!(first_token, second_token);

        // A stale (first) timeout arriving late must stay a no-op even
        // though it's told a popover is still open -- the poll-back must
        // not double-schedule on top of the newer timer.
        assert_eq!(
            state.on_autohide_timeout(first_token, false, true),
            AutohideAction::None
        );
        assert!(state.is_revealed());

        // Second consecutive tick: the popover is STILL open, so this must
        // poll again rather than giving up after a single cycle.
        let third_token = match state.on_autohide_timeout(second_token, false, true) {
            AutohideAction::ScheduleTimer { duration_ms, token } => {
                assert_eq!(duration_ms, 3000);
                token
            }
            other => panic!("expected ScheduleTimer (second poll-back), got {other:?}"),
        };
        assert!(state.is_revealed());
        assert_ne!(second_token, third_token);

        // The popover has actually closed by the time the third timer
        // fires: the bar hides normally.
        assert_eq!(
            state.on_autohide_timeout(third_token, false, false),
            AutohideAction::Hide
        );
        assert!(!state.is_revealed());
    }

    /// A registry-tracked dropdown blocking the hide must NOT trigger the
    /// walk-only poll-back -- `DropdownClosed` -> `on_popover_closed`
    /// already re-arms this case on its own, so polling here would just be
    /// redundant background work.
    #[test]
    fn test_registry_popover_blocks_without_polling() {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };

        assert_eq!(
            state.on_autohide_timeout(token, true, false),
            AutohideAction::None
        );
        assert!(state.is_revealed());
    }

    /// Drives a fresh, enabled `AutohideState` into the hidden state the
    /// same way the real flow would (leave, then time out), so trigger-hover
    /// tests start from a realistic precondition rather than poking private
    /// fields directly.
    fn hidden_state() -> AutohideState {
        let mut state = AutohideState::new(true, 3000);
        let token = match state.on_hover_leave(false) {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);
        assert!(!state.is_revealed());

        // Backdate past HIDE_ECHO_GRACE so callers exercising a later, real
        // `on_hover_enter`/`on_hover_motion` aren't caught by the same
        // near-instant-echo guard `test_hover_enter_echo_within_grace_is_ignored`
        // tests directly -- this helper represents the bar having been
        // hidden for a while, not this microsecond.
        state.backdate_hidden_at_past_grace();
        state
    }

    #[test]
    fn test_trigger_enter_reveals_without_claiming_cursor_inside() {
        let mut state = hidden_state();

        let action = state.on_trigger_enter();
        match action {
            AutohideAction::ScheduleTimer { duration_ms, .. } => assert_eq!(duration_ms, 3000),
            other => panic!("expected ScheduleTimer, got {other:?}"),
        }
        assert!(state.is_revealed());
        assert!(!state.is_cursor_inside());
    }

    #[test]
    fn test_trigger_enter_without_real_hover_rehides_after_timeout() {
        let mut state = hidden_state();

        let token = match state.on_trigger_enter() {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };

        // Pointer never actually reaches the bar's own hitbox. Without this
        // fallback timeout, the bar would stay revealed forever.
        assert_eq!(state.on_timeout(token, false), AutohideAction::Hide);
        assert!(!state.is_revealed());
    }

    #[test]
    fn test_trigger_enter_superseded_by_real_hover_enter() {
        let mut state = hidden_state();

        let stale_token = match state.on_trigger_enter() {
            AutohideAction::ScheduleTimer { token, .. } => token,
            other => panic!("expected ScheduleTimer, got {other:?}"),
        };

        // Pointer actually reaches the bar's own surface before the
        // trigger's fallback timer elapses.
        assert_eq!(state.on_hover_enter(), AutohideAction::CancelTimer);
        assert!(state.is_cursor_inside());

        // The trigger's fallback timer is now stale and must be ignored.
        assert_eq!(state.on_timeout(stale_token, false), AutohideAction::None);
        assert!(state.is_revealed());
    }

    #[test]
    fn test_trigger_enter_disabled_is_noop() {
        let mut state = AutohideState::new(false, 3000);
        assert_eq!(state.on_trigger_enter(), AutohideAction::None);
    }
}
