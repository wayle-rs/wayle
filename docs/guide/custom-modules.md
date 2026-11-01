---
title: Custom modules
outline: [2, 3]
---

# Custom modules

<div v-pre>

A custom module runs a shell command and renders its output in the bar. Define one under `[[modules.custom]]` in `config.toml` with a unique `id` and a `command`, then reference it in a layout as `custom-<id>`.

```toml
[[bar.layout]]
monitor = "*"
left = ["custom-cpu-temp"]

[[modules.custom]]
id = "cpu-temp"
command = "sensors | awk '/^Tctl:/ {sub(/\\+/,\"\",$2); print $2}'"
interval-ms = 5000
icon-name = "ld-thermometer-symbolic"
format = "{{ output }}"
```

The command runs through `sh -c`. Stderr is discarded and any single run is killed after 30 seconds.

## Modes

`mode` controls how the command runs. Default is `poll`.

### Poll

The command runs every `interval-ms` milliseconds (default 5000). Use it for commands that print a value and exit. Setting `interval-ms = 0` disables polling entirely; the command never runs on a schedule, so the module only updates from [`on-action`](#on-action) (paired with a `command` for the initial render, set a reasonable interval instead).

### Watch

`mode = "watch"` spawns the command once and reads its stdout line by line. Each line replaces the displayed value. Use it for long-running commands that stream updates (IPC subscribers, `tail -F`, log followers).

When the command exits, `restart-policy` decides what happens:

| Policy       | Behavior                                |
| ------------ | --------------------------------------- |
| `never`      | Leave the module with its last value    |
| `on-exit`    | Restart after any exit                  |
| `on-failure` | Restart only on non-zero exit or signal |

`restart-interval-ms` is the base delay. Successive rapid failures back off exponentially, capped at 30 seconds.

### No command

`command` is optional. A module without one renders a static icon and label and only reacts to clicks, which is useful for launcher-style buttons.

## Output

Plain-text output is bound to `{{ output }}`.

Output that starts with `{` or `[` is parsed as JSON. Every top-level field of a JSON object becomes a template variable, and `{{ output }}` stays available as the raw text. Array outputs are not unpacked; reach into them with `{{ items.0 }}`.

A handful of top-level JSON keys are reserved:

| Field        | Effect                                               |
| ------------ | ---------------------------------------------------- |
| `text`       | Overrides `format` entirely for this run             |
| `tooltip`    | Overrides `tooltip-format` for this run              |
| `percentage` | Integer 0-100 used to pick from `icon-names`         |
| `alt`        | Key used to pick from `icon-map`, beats `percentage` |
| `class`      | Whitespace-separated CSS classes added to the widget |

## Format templates

`format` and `tooltip-format` use [MiniJinja](https://docs.rs/minijinja), a Jinja2-compatible template engine.

```
{{ field }}                 read a top-level variable
{{ nested.field }}          dotted access into JSON objects
{{ items.0 }}               array index access
{{ val | default("?") }}    fallback for missing values
{{ val | upper }}           also: lower, trim, replace, length, round
```

`class-format` uses the same syntax. Its rendered output is split on whitespace, and each token becomes a CSS class on the module, in addition to any classes from the JSON `class` field.

## Icons

Three fields control the icon. In priority order:

1. `icon-map` matches on the JSON `alt` field.
2. `icon-names` is an array indexed by `percentage`. The range is split evenly:
   with four icons, 0-24% uses the first, 25-49% the second, and so on.
3. `icon-name` is the static fallback.

Set `icon-show = false` to hide the icon entirely.

## Click and scroll

Each event has its own shell command: `left-click`, `right-click`, `middle-click`, `scroll-up`, `scroll-down`. Scroll events are debounced over 50ms so a fast scroll coalesces into one action.

### on-action

If `on-action` is set, it runs after a click or scroll command finishes, and its output replaces the displayed value immediately. This is the pattern for interactive modules (volume, brightness, workspace switchers) where the user expects the bar to update before the next poll tick.

## Hiding the module

With `hide-if-empty = true`, the module and its gap in the layout disappear when the output is an empty string, `"0"`, or `"false"` (case-insensitive). Useful for notification-count badges and VPN indicators that should vanish when there is nothing to show.

## Examples

### Poll Mode: Disk usage with state icons

Emit JSON so the module can drive `icon-names` from `percentage` and still show a human-readable string in the label.

```toml
[[modules.custom]]
id = "disk"
interval-ms = 30000
command = '''
df -h / | awk 'NR==2 {
  sub(/%/, "", $5)
  printf "{\"percentage\":%s,\"used\":\"%s\",\"total\":\"%s\"}\n", $5, $3, $2
}'
'''
format = "{{ used }} / {{ total }}"
tooltip-format = "{{ percentage }}% used"
icon-names = [
  "ld-hard-drive-symbolic",
  "ld-hard-drive-symbolic",
  "ld-hard-drive-download-symbolic",
  "ld-triangle-alert-symbolic",
]
```

The command prints `{"percentage":71,"used":"603G","total":"899G"}`. `percentage` picks the icon (71% lands in the third bucket, the "download" icon). `{{ used }}` and `{{ total }}` fill the label. `tooltip-format` shows the percentage on hover.

### Watch Mode: Event-driven volume control

Wayle ships a built-in `volume` module that talks to PulseAudio directly. This custom-module version exists as an illustration of the watch-mode + `on-action` + `icon-map` pattern; in real life, use the built-in.

The command subscribes to PulseAudio events via `pactl subscribe`. Every time a sink changes, the script re-queries state and emits a JSON line. Watch mode keeps the command alive and reads one update per line, so the bar redraws the instant PulseAudio reports a change, no polling involved.

```toml
[[modules.custom]]
id = "volume"
mode = "watch"
restart-policy = "on-exit"
command = '''
emit_state() {
  vol=$(pactl get-sink-volume @DEFAULT_SINK@ \
    | grep -oP '\d+(?=%)' | head -1)
  mute=$(pactl get-sink-mute @DEFAULT_SINK@ | awk '{print $2}')
  alt=normal
  [ "$mute" = "yes" ] && alt=muted
  printf '{"percentage":%s,"alt":"%s"}\n' "$vol" "$alt"
}
emit_state
pactl subscribe | while read -r line; do
  case "$line" in *sink*) emit_state ;; esac
done
'''
scroll-up   = "pactl set-sink-volume @DEFAULT_SINK@ +5%"
scroll-down = "pactl set-sink-volume @DEFAULT_SINK@ -5%"
left-click  = "pactl set-sink-mute @DEFAULT_SINK@ toggle"
format = "{{ percentage }}%"
icon-names = [
  "ld-volume-x-symbolic",
  "ld-volume-1-symbolic",
  "ld-volume-2-symbolic",
]
icon-map = { muted = "ld-volume-x-symbolic" }
```

No `on-action` needed: the scroll and click commands change PulseAudio state, PulseAudio publishes the event, the subscribe loop picks it up, and the bar updates. This is the general shape to reach for whenever the underlying system can stream changes (D-Bus signals, `inotifywait`, Hyprland's `socat` stream, and similar).

For state with no event source (CPU temp, disk usage, weather), poll mode is the only option. Built-in modules cover most of the common event-driven surfaces: `battery`, `network`, `bluetooth`, `volume`, `microphone`, `media`, `hyprland-workspaces`. Use custom modules for the gaps.

Every field of `[[modules.custom]]` is listed at [`/config/modules/custom`](/config/modules/custom).

</div>
