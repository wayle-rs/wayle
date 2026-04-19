---
title: osd
outline: [2, 3]
---

# osd

<div v-pre>

On-screen display overlay for transient events like volume and brightness.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Show OSD overlays for volume, brightness, and keyboard toggles. |
| `position` | [`OsdPosition`](/config/types#osd-position) | `"bottom"` | Screen anchor position. |
| `duration` | u32 | `2500` | Auto-dismiss delay in milliseconds. |
| `monitor` | [`OsdMonitor`](/config/types#osd-monitor) | `"primary"` | Target monitor: "primary" or a connector name like "DP-1". |
| `margin` | [`Spacing`](/config/types#spacing) | `150` | Margin from screen edges. |
| `border` | bool | `true` | Show a border around the OSD. |

## Default configuration

```toml
[osd]
enabled = true
position = "bottom"
duration = 2500
monitor = "primary"
margin = 150.0
border = true
```


</div>
