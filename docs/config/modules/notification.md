---
title: notification
outline: [2, 3]
---

# notification

<div v-pre>

Notification center: icon in the bar, dropdown with history, DND toggle.

Add it to your layout with `notification`:

```toml
[[bar.layout]]
monitor = "*"
right = ["notification"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-name` | string | `"ld-bell-symbolic"` | Icon shown when no notifications and DND is off. |
| `icon-unread` | string | `"ld-bell-dot-symbolic"` | Icon shown when notifications exist. |
| `icon-dnd` | string | `"ld-bell-off-symbolic"` | Icon shown when Do Not Disturb is active. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display notification count label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |
| `blocklist` | array of string | `[]` | Glob patterns for app names whose notifications are blocked entirely. |
| `icon-source` | [`IconSource`](/config/types#icon-source) | `"automatic"` | How notification icons are resolved. |
| `popup-position` | [`PopupPosition`](/config/types#popup-position) | `"top-right"` | Screen position for popup notifications. |
| `popup-max-visible` | u32 | `5` | Maximum number of popups visible at once. |
| `popup-stacking-order` | [`StackingOrder`](/config/types#stacking-order) | `"newest-first"` | Order in which popups stack on screen. |
| `popup-duration` | u32 | `5000` | Maximum popup display duration in milliseconds. |
| `popup-hover-pause` | bool | `true` | Pause popup auto-dismiss timer on hover. |
| `popup-margin-x` | [`Spacing`](/config/types#spacing) | `0` | Horizontal margin from screen edges. |
| `popup-margin-y` | [`Spacing`](/config/types#spacing) | `0` | Vertical margin from screen edges. |
| `popup-gap` | [`Spacing`](/config/types#spacing) | `8` | Gap between stacked popups. |
| `popup-monitor` | [`PopupMonitor`](/config/types#popup-monitor) | `"primary"` | Target monitor: "primary" or a connector name like "DP-1". |
| `popup-close-behavior` | [`PopupCloseBehavior`](/config/types#popup-close-behavior) | `"dismiss"` | What happens when the close button on a popup is clicked. |
| `popup-shadow` | bool | `true` | Display drop shadow on popup cards. |
| `popup-urgency-bar` | [`UrgencyBarThreshold`](/config/types#urgency-bar-threshold) | `"low"` | Minimum urgency level that displays a colored urgency bar. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on notification count. |

::: details More about `blocklist`

Matched notifications are silently dropped.
Supports `*` (any characters) and `?` (single character).

Examples: `["notify-send", "*chromium*", "Vivaldi*"]`

:::

::: details More about `icon-source`

| Mode | Per-notification image | No image provided |
|------|----------------------|-------------------|
| `automatic` | Shows the image | Mapped icon |
| `mapped` | Ignored | Mapped icon |
| `application` | Shows the image | App's generic icon, then mapped fallback |

:::

::: details More about `popup-duration`

Applications may request a shorter timeout, which takes precedence.

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., many unread
notifications).

#### Example

```toml
[[modules.notification.thresholds]]
above = 5
icon-color = "status-warning"
label-color = "status-warning"

[[modules.notification.thresholds]]
above = 20
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:notification"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `"wayle notify dnd"` | Action on right click. Default toggles Do Not Disturb. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.notification]
icon-name = "ld-bell-symbolic"
icon-unread = "ld-bell-dot-symbolic"
icon-dnd = "ld-bell-off-symbolic"
border-show = false
border-color = "green"
icon-show = true
icon-color = "auto"
icon-bg-color = "green"
label-show = true
label-color = "green"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:notification"
right-click = "wayle notify dnd"
middle-click = ""
scroll-up = ""
scroll-down = ""
blocklist = []
icon-source = "automatic"
popup-position = "top-right"
popup-max-visible = 5
popup-stacking-order = "newest-first"
popup-duration = 5000
popup-hover-pause = true
popup-margin-x = 0.0
popup-margin-y = 0.0
popup-gap = 8.0
popup-monitor = "primary"
popup-close-behavior = "dismiss"
popup-shadow = true
popup-urgency-bar = "low"
thresholds = []
```


</div>
