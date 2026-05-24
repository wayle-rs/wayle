---
title: power
outline: [2, 3]
---

# power

<div v-pre>

Shutdown, reboot, and logout menu.

Add it to your layout with `power`:

```toml
[[bar.layout]]
monitor = "*"
right = ["power"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-name` | string | `"ld-power-symbolic"` | Icon name to display. |
| `border-show` | bool | `false` | Display border around button. |

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Icon container background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |
| `left-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on left click. |

## Default configuration

```toml
[modules.power]
icon-name = "ld-power-symbolic"
border-show = false
border-color = "red"
icon-color = "auto"
icon-bg-color = "red"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
left-click = ""
```


</div>
