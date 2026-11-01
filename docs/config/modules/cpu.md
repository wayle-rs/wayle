---
title: cpu
outline: [2, 3]
---

# cpu

<div v-pre>

CPU usage, frequency, and temperature.

Add it to your layout with `cpu`:

```toml
[[bar.layout]]
monitor = "*"
right = ["cpu"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `poll-interval-ms` | u64 | `2000` | Polling interval in milliseconds. |
| `temp-sensor` | string | `"auto"` | Temperature sensor label. |
| `format` | string | `"{{ percent }}%"` | Format string for the label. |
| `icon-name` | string | `"ld-cpu-symbolic"` | Icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on CPU usage percentage. |

::: details More about `poll-interval-ms`

Faster polling increases CPU usage.

:::

::: details More about `temp-sensor`

Use `"auto"` for automatic detection, or specify a
label (e.g., `"Tctl"`, `"Package id 0"`).

Run `sensors` to see available labels.

:::

::: details More about `format`

#### Placeholders

- `{{ percent }}` - CPU usage (0-100)
- `{{ freq_ghz }}` - Frequency of the busiest core (highest usage)
- `{{ avg_freq_ghz }}` - Average frequency across cores
- `{{ max_freq_ghz }}` - Maximum frequency among cores
- `{{ temp_c }}` - Temperature in Celsius (if available)
- `{{ temp_f }}` - Temperature in Fahrenheit (if available)

#### Examples

- `"{{ percent }}%"` - "45%"
- `"{{ percent }}% @ {{ freq_ghz }}GHz"` - "45% @ 3.2GHz"
- `"{{ percent }}% {{ temp_c }}C"` - "45% 62C"

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., high CPU usage).

#### Example

```toml
[[modules.cpu.thresholds]]
above = 70
icon-color = "status-warning"
label-color = "status-warning"

[[modules.cpu.thresholds]]
above = 90
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.cpu]
poll-interval-ms = 2000
temp-sensor = "auto"
format = "{{ percent }}%"
icon-name = "ld-cpu-symbolic"
border-show = false
border-color = "blue"
icon-show = true
icon-color = "auto"
icon-bg-color = "blue"
label-show = true
label-color = "blue"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
thresholds = []
```


</div>
