### Wayle Configuration - Notification Module Settings

## Shared Settings

settings-modules-notification-blocklist = Blocklist
    .description = Glob patterns for app names whose notifications are blocked

settings-modules-notification-icon-source = Icon Source
    .description = How notification icons are resolved

## Popup Settings

settings-modules-notification-popup-position = Popup Position
    .description = Corner of the screen where notification popups appear

settings-modules-notification-popup-max-visible = Max Visible
    .description = Maximum number of popups shown at once before stacking

settings-modules-notification-popup-stacking-order = Stacking Order
    .description = Whether new popups appear above or below existing ones

settings-modules-notification-popup-margin-x = Horizontal Margin
    .description = Distance from the left/right screen edge

settings-modules-notification-popup-margin-y = Vertical Margin
    .description = Distance from the top/bottom screen edge

settings-modules-notification-popup-gap = Popup Gap
    .description = Spacing between stacked notification popups

settings-modules-notification-popup-monitor = Popup Monitor
    .description = Which monitor shows popups: "primary" or a connector like "DP-1"

settings-modules-notification-popup-duration = Auto-Dismiss
    .description = How long popups stay visible before closing (ms)

settings-modules-notification-popup-hover-pause = Pause on Hover
    .description = Stop the auto-dismiss timer while hovering over a popup

settings-modules-notification-popup-close-behavior = Close Behavior
    .description = What happens when you close a popup (dismiss vs mark read)

settings-modules-notification-popup-urgency-bar = Urgency Bar
    .description = Show a colored bar on popups that meet a minimum urgency level

## Relative Time

notification-popup-time-just-now = Just now
notification-popup-time-minutes-ago = { $minutes }m ago
notification-popup-time-hours-ago = { $hours }h ago
