---
title: microphone
outline: [2, 3]
---

# microphone

<div v-pre>

Microphone input level and mute toggle.

Add it to your layout with `microphone`:

```toml
[[bar.layout]]
monitor = "*"
right = ["microphone"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-active` | string | `"ld-mic-symbolic"` | Icon shown when microphone is active (unmuted). |
| `icon-muted` | string | `"ld-mic-off-symbolic"` | Icon shown when microphone is muted. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display percentage label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on microphone volume percentage. |

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., high input gain).

#### Example

```toml
[[modules.microphone.thresholds]]
above = 70
icon-color = "status-warning"
label-color = "status-warning"

[[modules.microphone.thresholds]]
above = 90
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
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:audio"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `"wayle audio input-mute"` | Action on middle click. Default toggles input mute. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.microphone]
icon-active = "ld-mic-symbolic"
icon-muted = "ld-mic-off-symbolic"
border-show = false
border-color = "red"
icon-show = true
icon-color = "auto"
icon-bg-color = "red"
label-show = true
label-color = "red"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:audio"
right-click = ""
middle-click = "wayle audio input-mute"
scroll-up = ""
scroll-down = ""
thresholds = []
```


</div>
