---
title: osd
outline: [2, 3]
---

# osd

<div v-pre>

On-screen display overlay for transient events like volume and brightness.

The `auto-*` keys control whether a change *opens* the overlay. An overlay
that is already open always tracks its device's value, and that tracking
never restarts the dismiss timer — though a change that also re-triggers
the automatic display does.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Show OSD overlays for volume, brightness, and keyboard toggles. |
| `position` | [`OsdPosition`](/config/types#osd-position) | `"bottom"` | Screen anchor position. |
| `duration` | u32 | `2500` | Auto-dismiss delay in milliseconds. |
| `monitor` | [`OsdMonitor`](/config/types#osd-monitor) | `"primary"` | Target monitor: "primary" or a connector name like "DP-1". |
| `margin` | [`Spacing`](/config/types#spacing) | `150` | Margin from screen edges. |
| `border` | bool | `true` | Show a border around the OSD. |
| `layer` | [`Layer`](/config/types#layer) | `"overlay"` | Layer-shell layer the OSD is placed on. |

::: details More about `layer`

When `general.tearing-mode` is enabled, `overlay` is demoted to `top`
to allow fullscreen tearing.

:::

## Automatic display

| Field | Type | Default | Description |
|---|---|---|---|
| `auto-speaker` | bool | `true` | Show the speaker OSD automatically when output volume or mute changes. |
| `auto-microphone` | bool | `true` | Show the microphone OSD automatically when input volume or mute changes. |
| `auto-brightness` | bool | `true` | Show the brightness OSD automatically when display brightness changes. |
| `auto-toggles` | bool | `true` | Show the OSD automatically when caps, num, or scroll lock is pressed. |

::: details More about `auto-speaker`

Turn off to only show it on demand via `wayle osd speaker`.

:::

::: details More about `auto-microphone`

Turn off to only show it on demand via `wayle osd mic`.

:::

::: details More about `auto-brightness`

Turn off when an external daemon adjusts brightness continuously, so the
overlay isn't permanently on screen. `wayle osd brightness` still works.

:::

## Default configuration

```toml
[osd]
enabled = true
auto-speaker = true
auto-microphone = true
auto-brightness = true
auto-toggles = true
position = "bottom"
duration = 2500
monitor = "primary"
margin = 150.0
border = true
layer = "overlay"
```


</div>
