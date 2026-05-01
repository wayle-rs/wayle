---
title: wallpaper
outline: [2, 3]
---

# wallpaper

<div v-pre>

Wallpaper rendering, cycling, and per-monitor overrides.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `engine-enabled` | bool | `true` | Enable the awww wallpaper engine. Disable to use an external wallpaper tool while keeping color extraction and theming. |

## Transitions

| Field | Type | Default | Description |
|---|---|---|---|
| `transition-type` | [`TransitionType`](/config/types#transition-type) | `"simple"` | Transition animation type. |
| `transition-duration` | [`TransitionDuration`](/config/types#transition-duration) | `0.7` | Transition animation duration in seconds. |
| `transition-fps` | [`TransitionFps`](/config/types#transition-fps) | `60` | Transition animation frame rate. |

## Cycling

| Field | Type | Default | Description |
|---|---|---|---|
| `cycling-enabled` | bool | `false` | Enable automatic wallpaper cycling. |
| `cycling-directory` | string | `""` | Directory containing wallpaper images for cycling. |
| `cycling-mode` | [`CyclingMode`](/config/types#cycling-mode) | `"sequential"` | Wallpaper cycling order. |
| `cycling-interval-mins` | [`CyclingInterval`](/config/types#cycling-interval) | `15` | Time between wallpaper changes in minutes. |
| `cycling-same-image` | bool | `false` | Show the same cycling wallpaper on all monitors. Only affects shuffle mode since sequential already displays the same image. |

## Per-monitor overrides

| Field | Type | Default | Description |
|---|---|---|---|
| `monitors` | array of [`MonitorWallpaperConfig`](/config/types#monitor-wallpaper-config) | `[]` | Per-monitor wallpaper and fit mode settings. Each entry targets a monitor by connector name. See [`MonitorWallpaperConfig`] for the available fields. |

::: details More about `monitors`

#### Example

```toml
[[wallpaper.monitors]]
name = "DP-1"
wallpaper = "/home/me/pictures/wall-primary.png"
fit-mode = "fill"

[[wallpaper.monitors]]
name = "HDMI-1"
wallpaper = "/home/me/pictures/wall-secondary.png"
fit-mode = "fit"
```

:::

## Default configuration

```toml
[wallpaper]
engine-enabled = true
transition-type = "simple"
transition-duration = 0.699999988079071
transition-fps = 60
cycling-enabled = false
cycling-directory = ""
cycling-mode = "sequential"
cycling-interval-mins = 15
cycling-same-image = false
monitors = []
```


</div>
