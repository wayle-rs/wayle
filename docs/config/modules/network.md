---
title: network
outline: [2, 3]
---

# network

<div v-pre>

Network connection status with a dropdown for switching connections.

Add it to your layout with `network`:

```toml
[[bar.layout]]
monitor = "*"
right = ["network"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `wifi-disabled-icon` | string | `"cm-wireless-disabled-symbolic"` | WiFi icon when disabled. |
| `wifi-acquiring-icon` | string | `"cm-wireless-acquiring-symbolic"` | WiFi icon when connecting. |
| `wifi-offline-icon` | string | `"cm-wireless-offline-symbolic"` | WiFi icon when disconnected. |
| `wifi-connected-icon` | string | `"cm-wireless-connected-symbolic"` | WiFi icon when connected but signal strength unavailable. |
| `wifi-signal-icons` | array of string | `[...]` | WiFi signal strength icons from weak to excellent. |
| `wired-connected-icon` | string | `"cm-wired-symbolic"` | Wired icon when connected. |
| `wired-acquiring-icon` | string | `"cm-wired-acquiring-symbolic"` | Wired icon when connecting. |
| `wired-disconnected-icon` | string | `"cm-wired-disconnected-symbolic"` | Wired icon when disconnected. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display connection label (SSID for WiFi, "Wired" for ethernet). |
| `label-max-length` | u32 | `15` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `wifi-signal-icons`

The signal percentage maps to icons: 0-25% uses icons\[0\], 26-50% uses
icons\[1\], etc.

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:network"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.network]
wifi-disabled-icon = "cm-wireless-disabled-symbolic"
wifi-acquiring-icon = "cm-wireless-acquiring-symbolic"
wifi-offline-icon = "cm-wireless-offline-symbolic"
wifi-connected-icon = "cm-wireless-connected-symbolic"
wifi-signal-icons = [
    "cm-wireless-signal-weak-symbolic",
    "cm-wireless-signal-ok-symbolic",
    "cm-wireless-signal-good-symbolic",
    "cm-wireless-signal-excellent-symbolic",
]
wired-connected-icon = "cm-wired-symbolic"
wired-acquiring-icon = "cm-wired-acquiring-symbolic"
wired-disconnected-icon = "cm-wired-disconnected-symbolic"
border-show = false
border-color = "accent"
icon-show = true
icon-color = "auto"
icon-bg-color = "accent"
label-show = true
label-color = "accent"
label-max-length = 15
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:network"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
