---
title: volume
outline: [2, 3]
---

# volume

<div v-pre>

Output volume control with a dropdown for device and app volumes.

Add it to your layout with `volume`:

```toml
[[bar.layout]]
monitor = "*"
right = ["volume"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `level-icons` | array of string | `[...]` | Icons for volume levels from low to maximum. |
| `icon-muted` | string | `"ld-volume-x-symbolic"` | Icon shown when audio output is muted. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display percentage label. |
| `format` | string | `"{{ percent }}%"` | Format string for the label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on volume percentage. |

::: details More about `level-icons`

The percentage is divided evenly among icons. With 3 icons:
1-33% uses icons\[0\], 34-66% uses icons\[1\], 67-100% uses icons\[2\].

:::

::: details More about `format`

#### Placeholders

- `{{ percent }}` - Volume (0-100)

#### Examples

- `"{{ percent }}%"` - "45%"

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., boosted volume).

#### Example

```toml
[[modules.volume.thresholds]]
above = 100
icon-color = "status-warning"
label-color = "status-warning"

[[modules.volume.thresholds]]
above = 130
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"red"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:audio"` | Action on left click. Default opens the audio dropdown. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `"wayle audio output-mute"` | Action on middle click. Default toggles mute. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Dropdown

| Field | Type | Default | Description |
|---|---|---|---|
| `dropdown-app-icons` | [`AppIconSource`](/config/types#app-icon-source) | `"mapped"` | Icon source for app volume entries in the audio dropdown. |

## Default configuration

```toml
[modules.volume]
level-icons = [
    "ld-volume-symbolic",
    "ld-volume-1-symbolic",
    "ld-volume-2-symbolic",
]
icon-muted = "ld-volume-x-symbolic"
border-show = false
border-color = "red"
icon-show = true
icon-color = "auto"
icon-bg-color = "red"
label-show = true
label-color = "red"
format = "{{ percent }}%"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:audio"
right-click = ""
middle-click = "wayle audio output-mute"
scroll-up = ""
scroll-down = ""
dropdown-app-icons = "mapped"
thresholds = []
```


</div>
