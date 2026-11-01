---
title: custom
outline: [2, 3]
---

# custom

<div v-pre>

User-defined module that runs a shell command and renders the output in the bar.

Full walkthrough with examples at <https://wayle.app/guide/custom-modules>.

Add it to your layout with `custom-<id>`:

```toml
[[bar.layout]]
monitor = "*"
right = ["custom-<id>"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `id` | string | required | Unique identifier for this module. |
| `command` | unknown | `null` | Shell command to execute. |
| `mode` | [`ExecutionMode`](/config/types#execution-mode) | `"poll"` | Execution mode for the command. |
| `interval-ms` | u64 | `5000` | Polling interval in milliseconds. |
| `format` | string | `"{{ output }}"` | Format string for the label using Jinja2 template syntax. |
| `tooltip-format` | unknown | `null` | Format string for the tooltip (hover text). |
| `hide-if-empty` | bool | `false` | Hide module when output is empty, "0", or "false". |
| `class-format` | unknown | `null` | Format string for dynamic CSS classes. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `0` | Maximum label length in characters before truncation. |
| `border-show` | bool | `false` | Display border around button. |
| `on-action` | unknown | `null` | Shell command to run after any click/scroll action completes. |

::: details More about `id`

Referenced in bar layouts as `custom-<id>`. Must be unique across
all custom module definitions.

#### Example

```toml
[[modules.custom]]
id = "gpu-temp"

# Reference in layout:
# layout = ["custom-gpu-temp", "clock"]
```

:::

::: details More about `command`

The command runs via `sh -c` and should output to stdout.
Stderr is discarded. Commands have a 30-second timeout.

#### Output Parsing

- If output starts with `{` or `[`: parsed as JSON
- Otherwise: treated as plain text

#### Behavior by Mode

- **poll**: Executed every `interval-ms` milliseconds
- **watch**: Spawned once, each stdout line triggers a display update.
  Restarts are controlled by `restart-policy`.

:::

::: details More about `mode`

| Mode | Behavior |
|------|----------|
| `poll` | Run command every `interval-ms` (default) |
| `watch` | Spawn long-running process, update on each stdout line |

Use `poll` for commands that return current state and exit.
Use `watch` for commands that stream updates (e.g., `pactl subscribe`).

:::

::: details More about `interval-ms`

Only applies to `poll` mode. Ignored in `watch` mode.

Set to `0` for manual polling mode: no timer is started. In manual
mode, the command still runs once at startup.

:::

::: details More about `format`

#### Variables

- `{{ output }}` - Raw command output
- `{{ field }}` - JSON field access
- `{{ nested.field }}` - Nested field access
- `{{ items.0 }}` - Array index access

#### Filters

- `{{ val | default('fallback') }}` - Fallback for missing values
- `{{ "%02d" | format(val) }}` - Zero-padding
- `{{ val | upper }}`, `| lower`, `| trim` - String transforms

#### Examples

- `"{{ output }}°C"` - Plain text: "72°C"
- `"{{ percentage }}%"` - JSON field: "75%"
- `"{{ data.temp }}°C"` - Nested: "22°C"

If JSON output contains a `text` field, it overrides this format.

:::

::: details More about `tooltip-format`

Supports the same Jinja2 syntax as `format`. If not set, no tooltip is shown.
If JSON output contains a `tooltip` field, it overrides this format.

#### Example

```toml
format = "{{ percentage }}%"
tooltip-format = "Volume: {{ percentage }}% on {{ device }}"
```

:::

::: details More about `hide-if-empty`

When enabled, the module (including its gap in the bar layout) is
completely hidden if the output indicates an empty/disabled state.

:::

::: details More about `class-format`

Supports the same Jinja2 syntax as `format`. The formatted result is
split on whitespace and each word is added as a CSS class.

Combined with the `class` field from JSON output (if present).

#### Example

```toml
class-format = "volume-{{ alt }}"
# Output: {"alt": "muted"} → adds class "volume-muted"
```

:::

::: details More about `label-max-length`

When exceeded, label is truncated with ellipsis. Set to `0` to disable.

:::

::: details More about `on-action`

Executes after the action handler finishes, and its output updates
the display immediately. Useful for reflecting state changes without
waiting for the next poll interval.

#### Example

```toml
# Volume control with immediate feedback
scroll-up = "pactl set-sink-volume @DEFAULT_SINK@ +5%"
scroll-down = "pactl set-sink-volume @DEFAULT_SINK@ -5%"
on-action = '''
vol=$(pactl get-sink-volume @DEFAULT_SINK@ | grep -oP '\d+(?=%)' | head -1)
echo "{\"percentage\": $vol}"
'''
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon container background color. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Label text color. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Border color. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `""` | Action performed on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action performed on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action performed on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action performed on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action performed on scroll down. |

::: details More about `left-click`

Supports shell commands and dropdown toggles:
- `"pavucontrol"` — runs a shell command
- `"dropdown:audio"` — toggles the named dropdown panel
- `""` — no action (default)

If `on-action` is set, it runs after shell commands complete.

:::

::: details More about `right-click`

Supports shell commands and dropdown toggles (see `left-click`).
If `on-action` is set, it runs after shell commands complete.

:::

::: details More about `middle-click`

Supports shell commands and dropdown toggles (see `left-click`).
If `on-action` is set, it runs after shell commands complete.

:::

::: details More about `scroll-up`

Supports shell commands and dropdown toggles (see `left-click`).
Scroll events are debounced (50ms) to coalesce rapid scrolls.
If `on-action` is set, it runs after shell commands complete.

:::

::: details More about `scroll-down`

Supports shell commands and dropdown toggles (see `left-click`).
Scroll events are debounced (50ms) to coalesce rapid scrolls.
If `on-action` is set, it runs after shell commands complete.

:::

## Icons

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-name` | string | `""` | Static symbolic icon name. |
| `icon-names` | unknown | `null` | Array of icon names indexed by percentage (0-100). |
| `icon-map` | unknown | `null` | Map of icon names keyed by the `alt` field value. |
| `icon-show` | bool | `true` | Display module icon. |

::: details More about `icon-name`

Used when `icon-names` and `icon-map` don't provide a match.
Should be a symbolic icon name from the icon theme (e.g., `"ld-gpu-symbolic"`).

#### Example

```toml
icon-name = "ld-temperature-symbolic"
```

:::

::: details More about `icon-names`

Requires JSON output with a `percentage` field (0-100).
The array is divided evenly across the percentage range.

#### Resolution

For N icons, icon at index `floor(percentage * N / 101)` is selected:

- 4 icons: 0-24% → [0], 25-49% → [1], 50-74% → [2], 75-100% → [3]
- 5 icons: 0-19% → [0], 20-39% → [1], 40-59% → [2], 60-79% → [3], 80-100% → [4]

#### Example

```toml
icon-names = [
  "battery-empty-symbolic",
  "battery-caution-symbolic",
  "battery-low-symbolic",
  "battery-good-symbolic",
  "battery-full-symbolic"
]
```

:::

::: details More about `icon-map`

Requires JSON output with an `alt` field. The `alt` value is looked up
in this map. Use `"default"` as a fallback key.

**Priority**: `icon-map[alt]` takes precedence over `icon-names[percentage]`,
allowing state-specific icons to override percentage-based icons.

#### Example

```toml
# Volume with muted state override
icon-names = ["vol-0", "vol-33", "vol-66", "vol-100"]
icon-map = { "muted" = "audio-volume-muted-symbolic" }

# Output: {"percentage": 50, "alt": "muted"}
# Result: Uses "audio-volume-muted-symbolic" (alt match beats percentage)

# Output: {"percentage": 50}
# Result: Uses "vol-33" (percentage-based, no alt)
```

:::

## Restart

| Field | Type | Default | Description |
|---|---|---|---|
| `restart-policy` | [`RestartPolicy`](/config/types#restart-policy) | `"never"` | Restart policy for watch mode. |
| `restart-interval-ms` | [`RestartDelay`](/config/types#restart-delay) | `1000` | Base restart delay in milliseconds for watch mode. |

::: details More about `restart-policy`

Only applies to `watch` mode. Ignored in `poll` mode.

| Policy | Behavior |
|--------|----------|
| `never` | Do not restart after exit |
| `on-exit` | Restart after any exit |
| `on-failure` | Restart only after non-zero/signal exit |

:::

::: details More about `restart-interval-ms`

Only applies to `watch` mode. Ignored in `poll` mode.

Used when `restart-policy` is `on-exit` or `on-failure`.
Delay increases exponentially on rapid failures, capped at 30 seconds.

:::

## Default configuration

Required fields (must be set in your config): `id`.

```toml
[[modules.custom]]
mode = "poll"
interval-ms = 5000
restart-policy = "never"
restart-interval-ms = 1000
format = "{{ output }}"
hide-if-empty = false
icon-name = ""
icon-show = true
icon-color = "auto"
icon-bg-color = "auto"
label-show = true
label-color = "auto"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
border-show = false
border-color = "auto"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
