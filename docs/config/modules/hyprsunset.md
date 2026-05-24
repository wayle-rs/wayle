---
title: hyprsunset
outline: [2, 3]
---

# hyprsunset

<div v-pre>

Toggle for Hyprland's blue-light filter.

Add it to your layout with `hyprsunset`:

```toml
[[bar.layout]]
monitor = "*"
right = ["hyprsunset"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `format` | string | `"{{ status }}"` | Format string for the label. |
| `temperature` | u32 | `5000` | Color temperature in Kelvin when filter is enabled. Range: 1000-20000. |
| `gamma` | u32 | `100` | Display gamma percentage when filter is enabled. Range: 0-200. |
| `icon-off` | string | `"ld-sun-symbolic"` | Icon when filter is disabled (showing normal daylight colors). |
| `icon-on` | string | `"ld-moon-symbolic"` | Icon when filter is enabled (showing warm night colors). |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `format`

#### Placeholders

- `{{ status }}` - Filter status text (On, Off)
- `{{ temp }}` - Current temperature in Kelvin (shows "--" when disabled)
- `{{ gamma }}` - Current gamma percentage (shows "--" when disabled)
- `{{ config_temp }}` - Configured temperature (always available)
- `{{ config_gamma }}` - Configured gamma (always available)

#### Examples

- `"{{ status }}"` - "On"
- `"{{ temp }}K {{ gamma }}%"` - "4500K 80%"
- `"{{ status }} ({{ temp }}K)"` - "On (4500K)"

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
| `left-click` | [`ClickAction`](/config/types#click-action) | `":toggle"` | Action on left click. Default toggles blue light filter. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.hyprsunset]
format = "{{ status }}"
temperature = 5000
gamma = 100
icon-off = "ld-sun-symbolic"
icon-on = "ld-moon-symbolic"
border-show = false
border-color = "yellow"
icon-show = true
icon-color = "auto"
icon-bg-color = "yellow"
label-show = true
label-color = "yellow"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = ":toggle"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
