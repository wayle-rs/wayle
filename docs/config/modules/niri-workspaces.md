---
title: niri-workspaces
outline: [2, 3]
---

# niri-workspaces

<div v-pre>

Niri workspace indicators with click-to-switch.

Add it to your layout with `niri-workspaces`:

```toml
[[bar.layout]]
monitor = "*"
right = ["niri-workspaces"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `min-workspace-count` | u8 | `0` | Always-visible ceiling for numerically-named workspaces. |
| `monitor-specific` | bool | `true` | Show only workspaces on this bar's monitor. |
| `hide-trailing-empty` | bool | `true` | Hide niri's auto-allocated trailing empty workspace. |
| `display-mode` | [`DisplayMode`](/config/types#display-mode) | `"label"` | What identifies each workspace button. |
| `label-strategy` | [`LabelStrategy`](/config/types#label-strategy) | `"name-or-index"` | How to compose the workspace label when `display-mode = "label"`. |
| `urgent-show` | bool | `true` | Pulse animation on workspaces with urgent windows. |
| `urgent-mode` | [`UrgentMode`](/config/types#urgent-mode) | `"workspace"` | Where the urgent pulse is applied. |
| `active-indicator` | [`ActiveIndicator`](/config/types#active-indicator) | `"background"` | Visual indicator for the active workspace. |
| `divider` | string | `" "` | Text separator between workspace identity and app icons. |
| `app-icons-show` | bool | `false` | Show application icons for windows on each workspace. |
| `app-icons-dedupe` | bool | `true` | Deduplicate application icons within a workspace. |
| `app-icons-fallback` | string | `"ld-app-window-symbolic"` | Fallback icon for applications not matched by `app-icon-map`. |
| `app-icons-empty` | string | `"tb-minus-symbolic"` | Icon shown when a workspace has no application windows. |
| `icon-gap` | [`Spacing`](/config/types#spacing) | `0.3` | Gap between app icons within a workspace button. |
| `workspace-padding` | [`Spacing`](/config/types#spacing) | `0.5` | Padding for workspace content along the bar direction. |
| `icon-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale multiplier for workspace icons. |
| `label-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale multiplier for workspace labels and dividers. Range: 0.25-3.0. |
| `workspace-ignore` | array of string | `[]` | Workspaces to hide from the display. |
| `active-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Color for the active (visible on its output) workspace. |
| `occupied-color` | [`ColorValue`](/config/types#color-value) | `"fg-muted"` | Color for occupied workspaces (have windows but not active). |
| `empty-color` | [`ColorValue`](/config/types#color-value) | `"fg-subtle"` | Color for empty workspaces and placeholder slots. |
| `container-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Background color for the workspaces container. |
| `border-show` | bool | `false` | Display border around the workspaces container. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-default"` | Border color for the workspaces container. |
| `workspace-map` | [`WorkspaceMap`](/config/types#workspace-map) | `{}` | Per-workspace icon and color overrides, keyed by name or id-as-string. |
| `app-icon-map` | map of string | `{}` | Application icon mapping with glob pattern support. |
| `left-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:this"` | Action on left click. |
| `middle-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `""` | Action on middle click. |
| `right-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `""` | Action on right click. |
| `scroll-up` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:previous"` | Action on scroll up. |
| `scroll-down` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:next"` | Action on scroll down. |

::: details More about `min-workspace-count`

Workspaces whose name parses to an integer at or below this value
stay visible even when empty. Workspaces with non-numeric names
and those above the ceiling follow the normal empty/occupied
filtering. When `0` (default), no extra workspaces are forced
visible.

:::

::: details More about `monitor-specific`

When `true` (default), each bar shows only its own output's
workspaces. When `false`, all workspaces from every output are shown.

:::

::: details More about `hide-trailing-empty`

Niri keeps one empty workspace at the tail of every output for
dynamic allocation.

:::

::: details More about `display-mode`

- `label` (default): show the workspace label per `label-strategy`
- `icon`: show an icon from `workspace-map` (falls back to label if unmapped)
- `none`: show nothing — only app icons visible (if enabled)

:::

::: details More about `label-strategy`

- `index`: index only (`"1"`, `"2"`)
- `name-or-index` (default): name when set, index otherwise
- `name-only`: name only; unnamed workspaces show nothing
- `index-and-name`: `"1: web"` form; unnamed workspaces show the index alone

:::

::: details More about `urgent-mode`

- `workspace` (default): whole button pulses
- `application`: only the urgent app icon pulses, falling back to
  `workspace` when app icons are disabled

:::

::: details More about `app-icons-dedupe`

When `true`, one icon per unique `app_id`. When `false`, one icon
per window.

:::

::: details More about `icon-size`

Applies to identity icons and custom icons from `workspace-map`.
Range: 0.25-3.0.

:::

::: details More about `workspace-ignore`

Glob patterns matched against the workspace's name (if set), then
its index, then its stable id. Examples:
- `"scratch"` — hide the workspace named `scratch`
- `"1?"` — hide indices 10-19

:::

::: details More about `active-color`

In `background` indicator mode, also used as the button background.

:::

::: details More about `workspace-map`

#### Example

```toml
[modules.niri-workspaces.workspace-map]
web = { icon = "ld-globe-symbolic", color = "#4a90d9" }
terminal = { icon = "ld-terminal-symbolic" }
```

:::

::: details More about `app-icon-map`

Maps window `app_id` or title to symbolic icon names. Supports:
- No prefix: matches `app_id` (e.g. `"*firefox*"`)
- `app:` prefix: explicit `app_id` match (e.g. `"app:org.mozilla.*"`)
- `title:` prefix: matches window title (e.g. `"title:*YouTube*"`)

#### Example

```toml
[modules.niri-workspaces.app-icon-map]
"*firefox*" = "ld-globe-symbolic"
"title:*YouTube*" = "ld-youtube-symbolic"
```

:::

## Default configuration

```toml
[modules.niri-workspaces]
min-workspace-count = 0
monitor-specific = true
hide-trailing-empty = true
display-mode = "label"
label-strategy = "name-or-index"
urgent-show = true
urgent-mode = "workspace"
active-indicator = "background"
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
active-color = "accent"
occupied-color = "fg-muted"
empty-color = "fg-subtle"
container-bg-color = "bg-surface-elevated"
border-show = false
border-color = "border-default"
left-click = "focus:this"
middle-click = ""
right-click = ""
scroll-up = "focus:previous"
scroll-down = "focus:next"

[modules.niri-workspaces.workspace-map]

[modules.niri-workspaces.app-icon-map]
```


</div>
