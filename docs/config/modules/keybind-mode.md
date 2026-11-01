---
title: keybind-mode
outline: [2, 3]
---

# keybind-mode

<div v-pre>

Current keybind-mode indicator for modal compositors.

Add it to your layout with `keybind-mode`:

```toml
[[bar.layout]]
monitor = "*"
right = ["keybind-mode"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `format` | string | `"{{ mode }}"` | Format string for the label. |
| `icon-name` | string | `"ld-layers-symbolic"` | Symbolic icon name. |
| `auto-hide` | bool | `false` | Automatically hide module when no mode is active. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `format`

#### Placeholders

- `{{ mode }}` - Current keybind mode name (shows "default" when inactive)

#### Examples

- `"{{ mode }}"` - "resize"
- `"Mode: {{ mode }}"` - "Mode: resize"
- `"[{{ mode }}]"` - "[resize]"

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
[modules.keybind-mode]
format = "{{ mode }}"
icon-name = "ld-layers-symbolic"
auto-hide = false
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
```


</div>
