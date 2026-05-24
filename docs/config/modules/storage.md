---
title: storage
outline: [2, 3]
---

# storage

<div v-pre>

Disk usage for a mount point.

Add it to your layout with `storage`:

```toml
[[bar.layout]]
monitor = "*"
right = ["storage"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `poll-interval-ms` | u64 | `30000` | Polling interval in milliseconds. |
| `mount-point` | string | `"/"` | Mount point to monitor (e.g., `"/"`, `"/home"`). |
| `format` | string | `"{{ percent }}%"` | Format string for the label. |
| `icon-name` | string | `"ld-hard-drive-symbolic"` | Icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation. Set to 0 to disable. |
| `thresholds` | array of [`ThresholdEntry`](/config/types#threshold-entry) | `[]` | Dynamic color thresholds based on disk usage percentage. |

::: details More about `poll-interval-ms`

Faster polling increases CPU usage.

:::

::: details More about `format`

#### Placeholders

- `{{ percent }}` - Disk usage as integer (0-100)
- `{{ used_tib }}` - Used space in TiB
- `{{ used_gib }}` - Used space in GiB
- `{{ used_mib }}` - Used space in MiB
- `{{ used_auto }}` - Used space with auto unit (e.g., "128.5 GiB")
- `{{ total_tib }}` - Total space in TiB
- `{{ total_gib }}` - Total space in GiB
- `{{ total_mib }}` - Total space in MiB
- `{{ total_auto }}` - Total space with auto unit
- `{{ free_tib }}` - Free space in TiB
- `{{ free_gib }}` - Free space in GiB
- `{{ free_mib }}` - Free space in MiB
- `{{ free_auto }}` - Free space with auto unit
- `{{ filesystem }}` - Filesystem type (e.g., "ext4", "btrfs")

#### Examples

- `"{{ percent }}%"` - "45%"
- `"{{ used_auto }}/{{ total_auto }}"` - "128.5 GiB/512.0 GiB"
- `"{{ free_gib }} GiB free"` - "383.5 GiB free"

:::

::: details More about `thresholds`

Entries are checked in order; the last matching entry wins for each
color slot. Use `above` for high-value warnings (e.g., disk nearly full).

#### Example

```toml
[[modules.storage.thresholds]]
above = 70
icon-color = "status-warning"
label-color = "status-warning"

[[modules.storage.thresholds]]
above = 90
icon-color = "status-error"
label-color = "status-error"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"yellow"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
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
[modules.storage]
poll-interval-ms = 30000
mount-point = "/"
format = "{{ percent }}%"
icon-name = "ld-hard-drive-symbolic"
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
thresholds = []
```


</div>
