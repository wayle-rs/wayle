---
title: systray
outline: [2, 3]
---

# systray

<div v-pre>

System tray icons via the StatusNotifierItem protocol.

Add it to your layout with `systray`:

```toml
[[bar.layout]]
monitor = "*"
right = ["systray"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-scale` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale multiplier for tray item icons. |
| `item-gap` | [`Spacing`](/config/types#spacing) | `0.25` | Gap between tray items. |
| `internal-padding` | [`Spacing`](/config/types#spacing) | `0.5` | Padding at the ends of the container. |
| `blacklist` | array of string | `[]` | Glob patterns for tray items to hide. |
| `overrides` | array of [`TrayItemOverride`](/config/types#tray-item-override) | `[]` | Custom icon and color overrides. |
| `border-show` | bool | `false` | Display border around container. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-accent"` | Border color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Container background color token. |

::: details More about `internal-padding`

Applies to left/right edges for horizontal bars, or top/bottom edges
for vertical bars.

:::

::: details More about `blacklist`

Matches against item ID or title.
Example: `["*discord*", "Steam"]`

:::

::: details More about `overrides`

First matching override wins. Supports glob patterns.

```toml
[[module.systray.overrides]]
name = "*discord*"
icon = "si-discord-symbolic"
color = "blue"
```

:::

## Default configuration

```toml
[modules.systray]
icon-scale = 1.0
item-gap = 0.25
internal-padding = 0.5
blacklist = []
overrides = []
border-show = false
border-color = "border-accent"
button-bg-color = "bg-surface-elevated"
```


</div>
