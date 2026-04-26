---
title: hyprland-workspaces
outline: [2, 3]
---

# hyprland-workspaces

<div v-pre>

Hyprland workspace indicators with click-to-switch.

Add it to your layout with `hyprland-workspaces`:

```toml
[[bar.layout]]
monitor = "*"
right = ["hyprland-workspaces"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `min-workspace-count` | u8 | `0` | Minimum number of workspace buttons to display. |
| `monitor-specific` | bool | `true` | Show only workspaces belonging to the bar's monitor. |
| `show-special` | bool | `true` | Include special workspaces (scratchpads) in the display. |
| `highlight-active-on-other-monitor` | bool | `true` | Highlight workspace active on current moinitor and workspaces on other monitors with a different color. |
| `urgent-show` | bool | `true` | Pulse animation on workspaces with urgent windows. |
| `urgent-mode` | [`UrgentMode`](/config/types#urgent-mode) | `"workspace"` | Where the urgent pulse is applied. |
| `display-mode` | [`DisplayMode`](/config/types#display-mode) | `"label"` | What identifies each workspace button. |
| `label-use-name` | bool | `false` | Use workspace name instead of number when displaying labels. |
| `numbering` | [`Numbering`](/config/types#numbering) | `"absolute"` | How workspace numbers are displayed. |
| `divider` | string | `" "` | Text separator between workspace identity and app icons. |
| `app-icons-show` | bool | `false` | Show application icons for windows in each workspace. |
| `app-icons-dedupe` | bool | `true` | Deduplicate application icons within a workspace. |
| `app-icons-fallback` | string | `"ld-app-window-symbolic"` | Fallback icon for applications not matched by `app-icon-map`. |
| `app-icons-empty` | string | `"tb-minus-symbolic"` | Icon shown for empty workspaces when `app-icons-show` is enabled. |
| `icon-gap` | [`Spacing`](/config/types#spacing) | `0.3` | Gap between app icons within a workspace button. |
| `workspace-padding` | [`Spacing`](/config/types#spacing) | `0.5` | Padding for workspace content along the bar direction. |
| `icon-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale multiplier for workspace icons. |
| `label-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale multiplier for workspace labels and dividers. |
| `workspace-ignore` | array of string | `[]` | Workspaces to hide from the display. |
| `active-indicator` | [`ActiveIndicator`](/config/types#active-indicator) | `"background"` | Visual indicator for the active workspace. |
| `active-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Color for the active (focused) workspace. |
| `active-on-other-monitor-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Color for workspaces active (focused) on different monitors. |
| `occupied-color` | [`ColorValue`](/config/types#color-value) | `"fg-muted"` | Color for occupied workspaces (has windows but not focused). |
| `empty-color` | [`ColorValue`](/config/types#color-value) | `"fg-subtle"` | Color for empty workspaces. |
| `container-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Background color for the workspaces container. |
| `border-show` | bool | `false` | Display border around the workspaces container. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-default"` | Border color for the workspaces container. |
| `workspace-map` | [`WorkspaceMap`](/config/types#workspace-map) | `{}` | Per-workspace icon and color overrides. |
| `app-icon-map` | map of string | `{}` | Application icon mapping with glob pattern support. |

::: details More about `min-workspace-count`

When set to 0 (default), only active and occupied workspaces are shown.
When set to N, at least N buttons are always visible, with empty ones
using `empty-color` styling.

:::

::: details More about `monitor-specific`

When true, each bar shows only its monitor's workspaces.
When false, all workspaces from all monitors are shown.

:::

::: details More about `show-special`

Special workspaces have negative IDs in Hyprland.

:::

::: details More about `highlight-active-on-other-monitor`

This option only has an effect when `monitor-specific` is false.
- When false, the currently active workspace is highlighted with `active-color` on all monitors.
- When true, the workspace active on the current monitor is indicated with `active-color`, but all workspaces active on different monitors are indicated with `active-on-other-monitor-color`.
There will be no visual indication for the global active workspace.


::: details More about `urgent-show`

When a window requests attention (e.g., terminal bell), the workspace
button pulses until you switch to it.

:::

::: details More about `urgent-mode`

- `workspace`: Entire workspace pulses (default)
- `application`: Only the app icon(s) belonging to the urgent window
  pulse, falling back to `workspace` when app icons are disabled

:::

::: details More about `display-mode`

- `label`: Shows workspace number (or name if `label-use-name` is true)
- `icon`: Shows icon from `workspace-map` (falls back to label if unmapped)
- `none`: Shows nothing - only app icons visible

:::

::: details More about `label-use-name`

Only applies when `display-mode = "label"` or as fallback for unmapped
workspaces in `display-mode = "icon"`.

:::

::: details More about `numbering`

- `absolute`: Show actual Hyprland workspace IDs (1, 2, 3, 4, 5, 6...)
- `relative`: Show numbers relative to monitor's starting workspace.
  If a monitor has workspaces 4, 5, 6 assigned, they display as 1, 2, 3.
  Useful when keybinds use per-monitor numbering.

:::

::: details More about `divider`

Only shown when both `display-mode` is not `none` and `app-icons-show`
is enabled. Common values: `"|"`, `"·"`, `"-"`.

:::

::: details More about `app-icons-show`

When enabled, displays icons for running applications.
Icons are resolved via `app-icon-map` configuration.

:::

::: details More about `app-icons-dedupe`

When true, shows only one icon per unique window class.
When false, shows an icon for every window.

:::

::: details More about `app-icons-empty`

When a workspace has no windows but is displayed (via `min-workspace-count`),
this icon appears as a placeholder.

:::

::: details More about `icon-gap`

Only applies to spacing between app icons.

:::

::: details More about `workspace-padding`

For horizontal bars, controls horizontal (left/right) padding.
For vertical bars, controls vertical (top/bottom) padding.

:::

::: details More about `icon-size`

Applies to workspace identity icons and custom icons from `workspace-map`.
Range: 0.25-3.0.

:::

::: details More about `label-size`

Applies to workspace number/name labels and the divider text.
Range: 0.25-3.0.

:::

::: details More about `workspace-ignore`

Glob patterns matching workspace IDs. Examples:
- `"10"` - hide workspace 10
- `"1?"` - hide workspaces 10-19

:::

::: details More about `active-color`

Applied to icons and labels. In `background` indicator mode,
also used as the button background.

:::

::: details More about `active-on-other-monitor-color`

Applied to icons and labels. In `background` indicator mode,
also used as the button background.
Only makes a difference when `highlight-active-on-other-monitor` is true.

:::

::: details More about `occupied-color`

Applied to icons and labels.

:::

::: details More about `empty-color`

Applied to the empty placeholder icon and labels.

:::

::: details More about `workspace-map`

Keys are workspace IDs (use negative for special workspaces).

#### Example

```toml
[modules.hyprland-workspaces.workspace-map]
1 = { icon = "ld-globe-symbolic", color = "#4a90d9" }
2 = { icon = "ld-terminal-symbolic" }
```

:::

::: details More about `app-icon-map`

Maps window class or title to symbolic icon names. Supports:
- No prefix: Matches window class (e.g., `"*firefox*"`)
- `class:` prefix: Explicit class match (e.g., `"class:org.mozilla.*"`)
- `title:` prefix: Matches window title (e.g., `"title:*YouTube*"`)

User mappings are merged with built-in defaults for common applications.

#### Example

```toml
[modules.hyprland-workspaces.app-icon-map]
"*firefox*" = "ld-globe-symbolic"
"title:*YouTube*" = "ld-youtube-symbolic"
```

:::

## Default configuration

```toml
[modules.hyprland-workspaces]
min-workspace-count = 0
monitor-specific = true
show-special = true
urgent-show = true
urgent-mode = "workspace"
display-mode = "label"
label-use-name = false
numbering = "absolute"
divider = " "
app-icons-show = false
app-icons-dedupe = true
app-icons-fallback = "ld-app-window-symbolic"
app-icons-empty = "tb-minus-symbolic"
icon-gap = 0.30000001192092896
workspace-padding = 0.5
icon-size = 1.0
label-size = 1.0
workspace-ignore = []
active-indicator = "background"
active-color = "accent"
occupied-color = "fg-muted"
empty-color = "fg-subtle"
container-bg-color = "bg-surface-elevated"
border-show = false
border-color = "border-default"

[modules.hyprland-workspaces.workspace-map]

[modules.hyprland-workspaces.app-icon-map]
```


</div>
