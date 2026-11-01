---
title: ram
outline: [2, 3]
---

# ram

<div v-pre>

Memory and swap usage.

Add it to your layout with `ram`:

```toml
[[bar.layout]]
monitor = "*"
right = ["ram"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `poll-interval-ms` | u64 | `5000` | Polling interval in milliseconds. |
| `format` | string | `"{{ percent }}%"` | Format string for the label. |
| `icon-name` | string | `"ld-memory-stick-symbolic"` | Icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on RAM usage percentage. |

::: details More about `poll-interval-ms`

Faster polling increases CPU usage.

:::

::: details More about `format`

#### Memory Placeholders

- `{{ percent }}` - Memory usage as integer (0-100)
- `{{ used_gib }}` - Used memory in GiB (e.g., "7.2")
- `{{ total_gib }}` - Total memory in GiB (e.g., "16.0")
- `{{ available_gib }}` - Available memory in GiB (e.g., "8.8")

#### Swap Placeholders

- `{{ swap_percent }}` - Swap usage as integer (0-100)
- `{{ swap_used_gib }}` - Used swap in GiB
- `{{ swap_total_gib }}` - Total swap in GiB

#### Examples

- `"{{ percent }}%"` - "45%"
- `"{{ used_gib }}/{{ total_gib }} GiB"` - "7.2/16.0 GiB"
- `"{{ percent }}% (Swap: {{ swap_percent }}%)"` - "45% (Swap: 12%)"

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., high memory usage).

#### Example

```toml
[[modules.ram.thresholds]]
above = 80
icon-color = "status-warning"
label-color = "status-warning"

[[modules.ram.thresholds]]
above = 95
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"green"` | Label text color token. |
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
[modules.ram]
poll-interval-ms = 5000
format = "{{ percent }}%"
icon-name = "ld-memory-stick-symbolic"
border-show = false
border-color = "green"
icon-show = true
icon-color = "auto"
icon-bg-color = "green"
label-show = true
label-color = "green"
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
