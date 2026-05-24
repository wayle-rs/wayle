---
title: separator
outline: [2, 3]
---

# separator

<div v-pre>

A vertical rule between bar modules.

Add it to your layout with `separator`:

```toml
[[bar.layout]]
monitor = "*"
right = ["separator"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `size` | u32 | `1` | Thickness of the separator line in pixels. |
| `length` | [`Spacing`](/config/types#spacing) | `1.5` | Length of the separator line. |
| `color` | [`ColorValue`](/config/types#color-value) | `"fg-subtle"` | Color of the separator line. |

## Default configuration

```toml
[modules.separator]
size = 1
length = 1.5
color = "fg-subtle"
```


</div>
