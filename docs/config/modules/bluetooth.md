---
title: bluetooth
outline: [2, 3]
---

# bluetooth

<div v-pre>

Bluetooth connection status with a dropdown for pairing and managing devices.

Add it to your layout with `bluetooth`:

```toml
[[bar.layout]]
monitor = "*"
right = ["bluetooth"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `disabled-icon` | string | `"ld-bluetooth-off-symbolic"` | Icon when Bluetooth is disabled or unavailable. |
| `disconnected-icon` | string | `"ld-bluetooth-symbolic"` | Icon when Bluetooth is on but no devices connected. |
| `connected-icon` | string | `"ld-bluetooth-connected-symbolic"` | Icon when devices are connected. |
| `searching-icon` | string | `"ld-bluetooth-searching-symbolic"` | Icon when scanning for devices. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display connection label (device name or count). |
| `label-max-length` | u32 | `15` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:bluetooth"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.bluetooth]
disabled-icon = "ld-bluetooth-off-symbolic"
disconnected-icon = "ld-bluetooth-symbolic"
connected-icon = "ld-bluetooth-connected-symbolic"
searching-icon = "ld-bluetooth-searching-symbolic"
border-show = false
border-color = "blue"
icon-show = true
icon-color = "auto"
icon-bg-color = "blue"
label-show = true
label-color = "blue"
label-max-length = 15
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:bluetooth"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
