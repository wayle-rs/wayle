---
title: world-clock
outline: [2, 3]
---

# world-clock

<div v-pre>

Multiple timezones shown together in a dropdown.

Add it to your layout with `world-clock`:

```toml
[[bar.layout]]
monitor = "*"
right = ["world-clock"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `format` | string | `"{{ tz('UTC', '%H:%M %Z') }}"` | Format string with embedded timezone blocks. |
| `icon-name` | string | `"ld-globe-symbolic"` | Symbolic icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `format`

Use `{{ tz('timezone', 'strftime') }}` to insert a formatted time.
Anything outside a placeholder stays as literal text.

#### Examples

| Format string | Renders as |
|---|---|
| `"{{ tz('UTC', '%H:%M %Z') }}"` | `14:30 UTC` |
| `"NYC {{ tz('America/New_York', '%H:%M') }}  TYO {{ tz('Asia/Tokyo', '%H:%M') }}"` | `NYC 09:30  TYO 23:30` |
| `"{{ tz('America/New_York', '%H:%M %Z') }} \| {{ tz('Europe/London', '%H:%M %Z') }}"` | `09:30 EST \| 14:30 GMT` |

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
| `left-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.world-clock]
format = "{{ tz('UTC', '%H:%M %Z') }}"
icon-name = "ld-globe-symbolic"
border-show = false
border-color = "yellow"
icon-show = true
icon-color = "auto"
icon-bg-color = "yellow"
label-show = true
label-color = "yellow"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
