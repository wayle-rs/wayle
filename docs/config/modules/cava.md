---
title: cava
outline: [2, 3]
---

# cava

<div v-pre>

Audio frequency bars visualising the output stream.

Add it to your layout with `cava`:

```toml
[[bar.layout]]
monitor = "*"
right = ["cava"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `bars` | [`BarCount`](/config/types#bar-count) | `20` | Number of frequency bars. |
| `framerate` | [`Framerate`](/config/types#framerate) | `60` | Visualization update rate in frames per second. |
| `stereo` | bool | `false` | Stereo channel visualization (splits bars between left and right). |
| `noise-reduction` | [`NormalizedF64`](/config/types#normalized-f64) | `0.65` | Noise reduction filter strength. |
| `monstercat` | f64 | `0` | Monstercat-style smoothing across adjacent bars (0.0 = off). |
| `waves` | u32 | `0` | Wave-style smoothing (0 = off). |
| `low-cutoff` | [`FrequencyHz`](/config/types#frequency-hz) | `50` | Low frequency cutoff in Hz. |
| `high-cutoff` | [`FrequencyHz`](/config/types#frequency-hz) | `17000` | High frequency cutoff in Hz. |
| `input` | [`CavaInput`](/config/types#cava-input) | `"pipe-wire"` | Audio capture backend. |
| `source` | string | `"auto"` | Audio source identifier ("auto" for automatic selection). |
| `style` | [`CavaStyle`](/config/types#cava-style) | `"bars"` | Visualization rendering style. |
| `direction` | [`CavaDirection`](/config/types#cava-direction) | `"normal"` | Bar growth direction. |
| `color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Bar color. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Module background color. |
| `bar-width` | u32 | `6` | Width of each frequency bar in pixels. |
| `bar-gap` | u32 | `1` | Gap between frequency bars in pixels. |
| `internal-padding` | [`Spacing`](/config/types#spacing) | `0.5` | Padding at the ends of the visualizer. |
| `border-show` | bool | `false` | Display border around the visualizer. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-accent"` | Border color. |
| `left-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.cava]
bars = 20
framerate = 60
stereo = false
noise-reduction = 0.65
monstercat = 0.0
waves = 0
low-cutoff = 50
high-cutoff = 17000
input = "pipe-wire"
source = "auto"
style = "bars"
direction = "normal"
color = "accent"
button-bg-color = "bg-surface-elevated"
bar-width = 6
bar-gap = 1
internal-padding = 0.5
border-show = false
border-color = "border-accent"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
