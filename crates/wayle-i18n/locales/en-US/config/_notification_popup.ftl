### Wayle Configuration - Notification Module Settings

## Shared Settings

settings-modules-notifications-blocklist = Blocklist
    .description = Glob patterns for app names whose notifications are blocked

settings-modules-notifications-icon-source = Icon Source
    .description = How notification icons are resolved

## Popup Settings

settings-modules-notifications-popup-position = Popup Position
    .description = Corner of the screen where notification popups appear

settings-modules-notifications-popup-max-visible = Max Visible
    .description = Maximum number of popups shown at once before stacking

settings-modules-notifications-popup-stacking-order = Stacking Order
    .description = Whether new popups appear above or below existing ones

settings-modules-notifications-popup-margin-x = Horizontal Margin
    .description = Distance from the left/right screen edge

settings-modules-notifications-popup-margin-y = Vertical Margin
    .description = Distance from the top/bottom screen edge

settings-modules-notifications-popup-gap = Popup Gap
    .description = Spacing between stacked notification popups

settings-modules-notifications-popup-monitor = Popup Monitor
    .description = Which monitor shows popups: "primary" or a connector like "DP-1"

settings-modules-notifications-popup-layer = Popup Layer
    .description = Layer-shell layer popup notifications are placed on. Tearing mode demotes overlay to top.

settings-modules-notifications-popup-duration = Auto-Dismiss
    .description = How long popups stay visible before closing (ms)

settings-modules-notifications-popup-hover-pause = Pause on Hover
    .description = Stop the auto-dismiss timer while hovering over a popup

settings-modules-notifications-popup-close-behavior = Close Behavior
    .description = What happens when you close a popup (dismiss vs mark read)

settings-modules-notifications-popup-urgency-bar = Urgency Bar
    .description = Show a colored bar on popups that meet a minimum urgency level

## Relative Time

notification-popup-time-just-now = Just now
notification-popup-time-minutes-ago = { $minutes }m ago
notification-popup-time-hours-ago = { $hours }h ago
