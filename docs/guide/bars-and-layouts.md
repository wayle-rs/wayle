---
title: Bars and layouts
---

# Bars and layouts

A bar layout is an entry in the `[[bar.layout]]` array. Each entry targets a monitor (or all of them) and describes what the bar renders there. Fields:

| Field                     | Role                                                                                      |
| ------------------------- | ----------------------------------------------------------------------------------------- |
| `monitor`                 | Connector name like `"DP-1"`, or `"*"` to match any monitor without a more specific entry |
| `left`, `center`, `right` | Module lists for the three sections                                                       |
| `extends`                 | Optional. Inherit from another entry by its `monitor` value                               |
| `show`                    | Optional. Set to `false` to hide the bar on that monitor                                  |

Inside `left`, `center`, and `right`, a module slot accepts three shapes, mixable within the same section:

| Shape          | TOML                                                    |
| -------------- | ------------------------------------------------------- |
| Plain module   | `"clock"`                                               |
| Classed module | `{ module = "clock", class = "primary" }`               |
| Named group    | `{ name = "status", modules = ["battery", "network"] }` |

See [`BarLayout`](/config/types#bar-layout) for the full variant examples and [`/config/bar`](/config/bar) for spacing, colors, and button styling.

## Minimal layout

One entry matching every monitor, filled with built-in modules:

```toml
[[bar.layout]]
monitor = "*"
left = ["dashboard"]
center = ["clock"]
right = ["battery", "network", "volume", "systray"]
```

`monitor` takes a connector name like `"DP-1"` or `"HDMI-A-1"`, or `"*"` to match any monitor without a more specific entry.

## Per-monitor overrides

A second entry with `extends` inherits from another layout. Any section the child omits (or leaves as an empty list) is filled in from the parent; any section the child sets replaces the parent's.

```toml
[[bar.layout]]
monitor = "HDMI-1"
left = ["dashboard"]
center = ["clock"]
right = ["battery", "network", "volume", "systray"]

[[bar.layout]]
monitor = "*"
extends = "HDMI-1"
right = ["volume", "systray"]
```

On `*`, `left` and `center` come from the `"HDMI-1"` layout; `right` is this entry's list. `extends` can point at any other monitor value, not only `"HDMI-1"`.

## Groups and classes

A **group** renders its modules inside a shared GTK container. The group's `name` becomes the container's CSS ID, addressable as `#name` in a stylesheet (custom CSS coming soon). Use groups when several modules should share a visual container (padding, background, border radius) targetable by one selector.

A **class** attaches to a single module instance. Use it when the same module appears twice on a bar and each instance needs different styling, or when one instance should diverge from the module's default look.

```toml
[[bar.layout]]
monitor = "DP-1"
left = ["dashboard"]
center = [{ module = "clock", class = "primary" }]
right = [{ name = "status", modules = ["battery", "network", "volume"] }]
```

Custom modules slot in the same way, referenced as `custom-<id>` where `<id>` matches an entry in `[[modules.custom]]`. See [custom modules](/guide/custom-modules).

## Hiding a bar

Set `show = false` on an entry to hide the bar on that monitor. No other fields are required.

```toml
[[bar.layout]]
monitor = "HDMI-2"
show = false
```
