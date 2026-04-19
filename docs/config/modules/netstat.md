---
title: netstat
outline: [2, 3]
---

# netstat

<div v-pre>

Network traffic counters (up/down rates).

Add it to your layout with `netstat`:

```toml
[[bar.layout]]
monitor = "*"
right = ["netstat"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `poll-interval-ms` | u64 | `2000` | Polling interval in milliseconds. |
| `interface` | string | `"auto"` | Network interface to monitor. |
| `format` | string | `"{{ down_auto }} {{ up_auto }}"` | Format string for the label. |
| `icon-name` | string | `"ld-activity-symbolic"` | Icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation. Set to 0 to disable. |

::: details More about `poll-interval-ms`

Faster polling increases CPU usage.

:::

::: details More about `interface`

Use `"auto"` to select the first active interface, or specify an
interface name like `"eth0"` or `"wlan0"`.

:::

::: details More about `format`

#### Download Placeholders

- `{{ down_kib }}` - Download speed in KiB/s
- `{{ down_mib }}` - Download speed in MiB/s
- `{{ down_gib }}` - Download speed in GiB/s
- `{{ down_auto }}` - Download speed with auto unit (e.g., "1.5 MiB/s")

#### Upload Placeholders

- `{{ up_kib }}` - Upload speed in KiB/s
- `{{ up_mib }}` - Upload speed in MiB/s
- `{{ up_gib }}` - Upload speed in GiB/s
- `{{ up_auto }}` - Upload speed with auto unit (e.g., "256 KiB/s")

#### Other Placeholders

- `{{ interface }}` - Interface name (e.g., "wlan0")

#### Examples

- `"{{ down_auto }} {{ up_auto }}"` - "1.5 MiB/s 256 KiB/s"
- `"D:{{ down_mib }} U:{{ up_mib }}"` - "D:1.5 U:0.2"
- `"{{ interface }}: {{ down_auto }}"` - "wlan0: 1.5 MiB/s"

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Label text color token. |
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
[modules.netstat]
poll-interval-ms = 2000
interface = "auto"
format = "{{ down_auto }} {{ up_auto }}"
icon-name = "ld-activity-symbolic"
border-show = false
border-color = "red"
icon-show = true
icon-color = "auto"
icon-bg-color = "red"
label-show = true
label-color = "red"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
