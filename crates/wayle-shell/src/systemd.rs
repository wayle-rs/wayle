//! systemd service-manager readiness notification.
//!
//! When Wayle is run as a `Type=notify` systemd unit (see
//! `resources/wayle.service`), the service manager keeps
//! `graphical-session.target` — and anything ordered `After=` it, such as
//! `xdg-desktop-autostart.target` — waiting until Wayle reports that it has
//! finished starting up. This lets autostarted apps reliably find Wayle's
//! notification buses already owned instead of racing them (a GApplication
//! picks its `GNotification` backend once, on first send, and caches it for
//! life).
//!
//! Readiness is a *bootstrap-level* concern, not a per-module one: it is
//! reported exactly once, after the last shell module's constructor has
//! returned (the end of [`Shell::init`](crate::shell)). The contract for
//! modules is therefore that their constructors block until they have
//! finished setting up.
//!
//! This is a no-op unless launched by a notify-enabled systemd unit:
//! [`sd_notify::notify`] returns `Ok(())` and does nothing when
//! `$NOTIFY_SOCKET` is unset, so it is safe under compositor autostart
//! (Hyprland `exec-once`, `dex`, …) and when run straight from a shell.

use sd_notify::NotifyState;
use tracing::{debug, warn};

/// Reports startup completion to the service manager (`READY=1`).
///
/// Call exactly once, after every shell module has been constructed. No-ops
/// when `$NOTIFY_SOCKET` is unset — i.e. whenever Wayle was not launched by a
/// `Type=notify` systemd unit.
pub(crate) fn notify_ready() {
    // `sd_notify::notify` leaves `$NOTIFY_SOCKET` set (it does not mutate the
    // environment, which would be unsound here — the shell is already
    // multithreaded). That is harmless: the unit uses `NotifyAccess=main`, so
    // the service manager ignores any notifications from child processes.
    match sd_notify::notify(&[NotifyState::Ready]) {
        Ok(()) => {
            debug!("reported startup readiness (no-op unless run as a Type=notify systemd unit)");
        }
        Err(err) => warn!(error = %err, "failed to notify service manager of readiness"),
    }
}
