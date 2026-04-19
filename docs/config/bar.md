---
title: bar
outline: [2, 3]
---

# bar

<div v-pre>

Bar chrome: per-monitor layout, spacing, colors, and button styling.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `layout` | array of [`BarLayout`](/config/types#bar-layout) | `[...]` | Per-monitor bar layouts. Each entry targets a monitor by connector name (e.g., `"DP-1"`) or `"*"` for all monitors. See [`BarLayout`] for the full shape, including layout inheritance via `extends`. |
| `scale` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Bar-specific scale multiplier for spacing, radius, and other bar elements. |
| `inset-edge` | [`Spacing`](/config/types#spacing) | `0` | Gap between bar and its attached screen edge. |
| `inset-ends` | [`Spacing`](/config/types#spacing) | `0` | Gap at the bar's ends. |
| `padding` | [`Spacing`](/config/types#spacing) | `0.35` | Internal spacing along bar thickness. |
| `padding-ends` | [`Spacing`](/config/types#spacing) | `0.5` | Internal spacing at bar ends. |
| `module-gap` | [`Spacing`](/config/types#spacing) | `0.5` | Gap between modules and groups on the bar. |
| `location` | [`Location`](/config/types#location) | `"top"` | Bar position on screen edge. |
| `background-opacity` | [`Percentage`](/config/types#percentage) | `100` | Bar background opacity (0-100). |
| `border-location` | [`BorderLocation`](/config/types#border-location) | `"none"` | Border placement for bar. |
| `border-width` | u8 | `1` | Border width for bar (pixels). |
| `rounding` | [`RoundingLevel`](/config/types#rounding-level) | `"none"` | Corner rounding level for the bar. |
| `shadow` | [`ShadowPreset`](/config/types#shadow-preset) | `"none"` | Shadow style for the bar. |

::: details More about `layout`

#### Example

```toml
[[bar.layout]]
monitor = "*"
left = ["dashboard"]
center = ["clock"]
right = ["battery", "network", "volume", "systray"]

[[bar.layout]]
monitor = "HDMI-1"
extends = "*"
right = ["volume", "systray"]
```

:::

::: details More about `inset-edge`

- **Orientation**: Distance from top (horizontal bar) or left (vertical bar)

:::

::: details More about `inset-ends`

- **Orientation**: Left/right (horizontal bar), top/bottom (vertical bar)

:::

::: details More about `padding`

- **Orientation**: Top/bottom (horizontal bar), left/right (vertical bar)

:::

::: details More about `padding-ends`

- **Orientation**: Left/right (horizontal bar), top/bottom (vertical bar)

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `bg` | [`ColorValue`](/config/types#color-value) | `"bg-surface"` | Bar background color. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-accent"` | Border color for the bar. |
| `button-group-background` | [`ColorValue`](/config/types#color-value) | `"bg-elevated"` | Background color for button groups. |
| `button-group-border-color` | [`ColorValue`](/config/types#color-value) | `"border-accent"` | Border color for button groups. |

## Buttons

| Field | Type | Default | Description |
|---|---|---|---|
| `button-variant` | [`BarButtonVariant`](/config/types#bar-button-variant) | `"block-prefix"` | Visual style variant for bar buttons. |
| `button-opacity` | [`Percentage`](/config/types#percentage) | `100` | Button opacity (0-100). |
| `button-bg-opacity` | [`Percentage`](/config/types#percentage) | `100` | Button background opacity (0-100). |
| `button-icon-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Button icon size. |
| `button-icon-padding` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Button icon container padding. Only applies to `block-prefix` and `icon-square` variants. |
| `button-label-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Button label text size. |
| `button-label-weight` | [`FontWeightClass`](/config/types#font-weight-class) | `"semibold"` | Button label font weight. |
| `button-label-padding` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Button label container padding. |
| `button-rounding` | [`RoundingLevel`](/config/types#rounding-level) | `"sm"` | Corner rounding level for the buttons in the bar. |
| `button-gap` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Gap between button icon and label. |
| `button-icon-position` | [`IconPosition`](/config/types#icon-position) | `"start"` | Icon position relative to label in bar buttons. |
| `button-border-location` | [`BorderLocation`](/config/types#border-location) | `"all"` | Border placement for bar buttons. |
| `button-border-width` | u8 | `1` | Border width for bar buttons (pixels). |
| `button-group-border-location` | [`BorderLocation`](/config/types#border-location) | `"none"` | Border placement for button groups. |
| `button-group-border-width` | u8 | `1` | Border width for button groups (pixels). |
| `button-group-padding` | [`Spacing`](/config/types#spacing) | `0` | Internal padding for button groups. |
| `button-group-module-gap` | [`Spacing`](/config/types#spacing) | `0.25` | Gap between modules within a group. |
| `button-group-opacity` | [`Percentage`](/config/types#percentage) | `100` | Button group opacity (0-100). |
| `button-group-rounding` | [`RoundingLevel`](/config/types#rounding-level) | `"sm"` | Corner rounding level for button groups. |

## Dropdowns

| Field | Type | Default | Description |
|---|---|---|---|
| `dropdown-shadow` | bool | `true` | Enable dropdown panel shadow. |
| `dropdown-opacity` | [`Percentage`](/config/types#percentage) | `100` | Dropdown panel opacity (0-100). |
| `dropdown-autohide` | bool | `true` | Close dropdown when clicking outside it. |
| `dropdown-freeze-label` | bool | `true` | Freeze the bar button label while its dropdown is open. |

::: details More about `dropdown-freeze-label`

Prevents the button from resizing mid-interaction, which keeps the
dropdown anchored in place.

:::

## Default configuration

```toml
[bar]
scale = 1.0
inset-edge = 0.0
inset-ends = 0.0
padding = 0.3499999940395355
padding-ends = 0.5
module-gap = 0.5
location = "top"
bg = "bg-surface"
background-opacity = 100
border-location = "none"
border-width = 1
border-color = "border-accent"
rounding = "none"
shadow = "none"
button-variant = "block-prefix"
button-opacity = 100
button-bg-opacity = 100
button-icon-size = 1.0
button-icon-padding = 1.0
button-label-size = 1.0
button-label-weight = "semibold"
button-label-padding = 1.0
button-rounding = "sm"
button-gap = 1.0
button-icon-position = "start"
button-border-location = "all"
button-border-width = 1
button-group-border-location = "none"
button-group-border-width = 1
button-group-padding = 0.0
button-group-module-gap = 0.25
button-group-background = "bg-elevated"
button-group-opacity = 100
button-group-border-color = "border-accent"
button-group-rounding = "sm"
dropdown-shadow = true
dropdown-opacity = 100
dropdown-autohide = true
dropdown-freeze-label = true
```


</div>
