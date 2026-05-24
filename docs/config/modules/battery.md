---
title: battery
outline: [2, 3]
---

# battery

<div v-pre>

Battery level, charging state, and a dropdown with power-profile controls.

Add it to your layout with `battery`:

```toml
[[bar.layout]]
monitor = "*"
right = ["battery"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `level-icons` | array of string | `[...]` | Icons for battery levels from empty to full. |
| `charging-icon` | string | `"md-battery_android_frame_bolt-symbolic"` | Icon shown when battery is charging. |
| `alert-icon` | string | `"md-battery_android_alert-symbolic"` | Icon shown when battery is not present or in an error state. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display percentage label. |
| `format` | string | `"{{ percent }}%"` | Format string for the label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on battery percentage. |

::: details More about `level-icons`

The percentage is divided evenly among icons. With 5 icons:
0-20% uses icons\[0\], 21-40% uses icons\[1\], etc.

:::

::: details More about `format`

#### Placeholders

- `{{ percent }}` - Battery level (0-100)

#### Examples

- `"{{ percent }}%"` - "45%"

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `below` for low-value warnings (e.g., low battery).

#### Example

```toml
[[modules.battery.thresholds]]
below = 40
icon-color = "status-warning"

[[modules.battery.thresholds]]
below = 20
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"yellow"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"yellow"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"yellow"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:battery"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.battery]
level-icons = [
    "md-battery_android_0-symbolic",
    "md-battery_android_frame_1-symbolic",
    "md-battery_android_frame_2-symbolic",
    "md-battery_android_frame_3-symbolic",
    "md-battery_android_frame_4-symbolic",
    "md-battery_android_frame_5-symbolic",
    "md-battery_android_frame_6-symbolic",
    "md-battery_android_frame_full-symbolic",
]
charging-icon = "md-battery_android_frame_bolt-symbolic"
alert-icon = "md-battery_android_alert-symbolic"
border-show = false
border-color = "yellow"
icon-show = true
icon-color = "auto"
icon-bg-color = "yellow"
label-show = true
label-color = "yellow"
format = "{{ percent }}%"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:battery"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
thresholds = []
```


</div>
