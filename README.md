<p align="center">
  <img src="assets/wayle.svg" width="200" alt="Wayle">
</p>

# Wayle

[![CI](https://img.shields.io/github/actions/workflow/status/wayle-rs/wayle/ci.yml?branch=master)](https://github.com/wayle-rs/wayle/actions)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/wayle-rs/wayle/blob/master/LICENSE)

A configurable desktop shell for Wayland compositors. Built in Rust with GTK4
and Relm4. Compositor-agnostic successor to HyprPanel.

## Progress

### Core Infrastructure

- [x] **Configuration System** - Reactive TOML config with schema validation
- [x] **CLI Interface** - Complete command-line management interface
- [x] **Documentation Generator** - Auto-generated config docs from schemas

### Services

- [x] **MPRIS**
- [x] **PulseAudio**
- [x] **Network**
- [x] **Bluetooth**
- [x] **Battery**
- [x] **Notification Daemon**
- [x] **Power Profiles**
- [x] **System Tray**
    - [x] GTK4 Adapter
- [x] Hyprland
- [x] **Cava**

### UI Components

- [x] **Component Library** - Base Relm4 widgets and containers
- [x] **Bar Modules**:
    - [x] Battery
    - [x] Media
    - [x] Volume
    - [x] Network
    - [x] Bluetooth
    - [x] Clock
    - [x] Microphone
    - [x] System tray
    - [x] Notification
    - [x] Dashboard
    - [x] Netstat
    - [x] RAM
    - [x] CPU
    - [x] CPU Temp
    - [x] Storage
    - [x] Separator
    - [x] Power
    - [x] World clock
    - [x] Weather
    - [x] Idle Inhibit
    - [x] Keyboard input
    - [x] Hyprland Window title
    - [x] Hyprland submap
    - [x] Hyprsunset
    - [x] Hyprland workspaces
    - [x] Custom Modules
    - [x] Cava

#### Scoped out

- **Updates**
    - Too much surface area and distro coupling
    - Will be achievable easily via custom modules

### Dropdown Interfaces

- [x] **Audio Panel**
- [x] **Network Panel**
- [x] **Bluetooth Panel**
- [x] **Battery Panel**
- [x] **Media Panel**
- [x] **Weather Panel**
- [x] **Calendar Panel**
- [x] **Dashboard**
- [x] **Notifications Panel**

### Additional Features

- [x] **Notifications**
- [x] **OSD**
- [x] **Settings Dialog**

NOTE: 04/14/2026 - The Settings Dialog has been completed which was the last
step. HyprPanel will be archived shortly. Support for Niri, Sway and Mango
upcoming...

## Configuration

Configuration lives in `~/.config/wayle/config.toml` with live reloading.

```toml
[styling]
theme-provider = "wayle"

[styling.palette]
bg = "#16161e"
fg = "#c0caf5"
primary = "#7aa2f7"

[bar]
scale = 1
location = "top"
rounding = "sm"

[[bar.layout]]
monitor = "*"
left = ["clock"]
center = ["media"]
right = ["battery"]

[modules.clock]
format = "%H:%M"
icon-show = true
label-show = true
```

Config files can be split and imported for better organization:

```toml
# config.toml
imports = ["colors.toml", "modules/bar.toml"]

[bar]
location = "top"
```

CLI commands can also be used to modify, get or reset any property:

```bash
wayle config get bar.scale
wayle config set bar.location bottom
wayle config reset bar.scale
```

Once the project is finished, documentation will be added for all configurable
properties, in addition to having a settings GUI. Until then you can run the
following command to generate a reference config `config.toml.example` in your
config directory:

```bash
wayle config default
```

Editor intellisense is available via JSON Schema. Install
[Tombi](https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi)
for VSCode or the `tombi` LSP for Neovim. The schema is generated automatically
on startup.

This will give you auto-complete, config validation and other nice QoL features
for your config.toml (and other toml files).

```bash
wayle config schema
```

## Installation

### Arch Linux (AUR)

```bash
yay -S wayle-bin
```

Then start the shell:

```bash
wayle shell
```

### From source

Install Rust via [rustup](https://rustup.rs), then:

```bash
git clone https://github.com/wayle-rs/wayle
cd wayle
cargo install --path wayle
wayle icons setup
wayle panel start
```

## Icons

Wayle uses GTK symbolic icons that support CSS color theming.

To manually manage icons:

```bash
# Install bundled icons (automatic on first launch)
wayle icons setup

# See all available icon sources and their prefixes
wayle icons sources

# List installed icons
wayle icons list

# Filter installed icons by source prefix (e.g. ld, tb, si, md)
wayle icons list --source ld

# Interactive fuzzy search of installed icons (requires fzf)
wayle icons list --interactive

# Install additional icons from CDN sources
wayle icons install tabler home settings bell
wayle icons install simple-icons firefox spotify

# See all available sources
wayle icons install --help
```

Icons are installed to `~/.local/share/wayle/icons/` as GTK symbolic icons. For
remote icon discovery, browse each source website shown in `wayle icons sources`
and use the icon slug with `wayle icons install <source> <slug...>`.

## Custom Modules

Custom modules run shell commands and display the output in the bar. Define one
in your config and add it to your layout with the `custom-` prefix:

```toml
[[bar.layout]]
monitor = "*"
right = ["custom-gpu-temp", "clock"]

[[modules.custom]]
id = "gpu-temp"
command = "nvidia-smi --query-gpu=temperature.gpu --format=csv,noheader,nounits"
interval-ms = 5000
format = "{{ output }}°C"
icon-name = "ld-thermometer-symbolic"
```

The `command` runs via `sh -c`. Plain text output is available as `{{ output }}`
in `format`. If the output starts with `{` or `[`, it's parsed as JSON and
fields are available directly: `{{ temperature }}`, `{{ nested.value }}`, etc.

### Execution Modes

**Poll** (default) runs the command every `interval-ms` milliseconds:

```toml
# default, can be omitted
mode = "poll"
# every 5 seconds
interval-ms = 5000
```

**Watch** spawns the command once and updates the display on each line of
stdout. Good for commands that stream events like `pactl subscribe` or
`inotifywait`:

```toml
[[modules.custom]]
id = "volume"
mode = "watch"
command = '''
pactl subscribe | while read -r line; do
  if [[ "$line" == *"sink"* ]]; then
    vol=$(pactl get-sink-volume @DEFAULT_SINK@ | grep -oP '\d+(?=%)' | head -1)
    echo "{\"percentage\": $vol}"
  fi
done
'''
format = "{{ percentage }}%"
restart-policy = "on-failure"
```

If a watch process exits, `restart-policy` controls what happens:

- `never` (default) - stay dead
- `on-exit` - restart after any exit
- `on-failure` - restart only on non-zero exit codes

The restart delay starts at `restart-interval-ms` (default 1000ms) and doubles
on each rapid failure, capping at 30 seconds.

### Dynamic Icons

If your command outputs JSON with a `percentage` field (0-100), you can map it
to an array of icons. The array is divided evenly across the range:

```toml
[[modules.custom]]
id = "battery"
command = '''
cap=$(cat /sys/class/power_supply/BAT0/capacity)
echo "{\"percentage\": $cap}"
'''
interval-ms = 30000
format = "{{ percentage }}%"
icon-names = [
  "ld-battery-warning-symbolic",
  "ld-battery-low-symbolic",
  "ld-battery-medium-symbolic",
  "ld-battery-full-symbolic"
]
```

4 icons means: 0-24% picks the first, 25-49% the second, 50-74% the third,
75-100% the fourth.

For state-based icons, output an `alt` field and use `icon-map`:

```toml
icon-map = { muted = "ld-volume-off-symbolic", default = "ld-volume-2-symbolic" }
```

If both `alt` and `percentage` are present, `icon-map` wins. The full priority
is: `icon-map[alt]` > `icon-names[percentage]` > `icon-map["default"]` >
`icon-name`.

### Click Actions

Each interaction type has its own command:

```toml
left-click = "pavucontrol"
scroll-up = "pactl set-sink-volume @DEFAULT_SINK@ +5%"
scroll-down = "pactl set-sink-volume @DEFAULT_SINK@ -5%"
```

By default, the display won't update until the next poll. To refresh immediately
after an action, add `on-action` - its output updates the display right away:

```toml
on-action = '''
vol=$(pactl get-sink-volume @DEFAULT_SINK@ | grep -oP '\d+(?=%)' | head -1)
echo "{\"percentage\": $vol}"
'''
```

Scroll events are debounced (50ms) so rapid scrolling doesn't fire dozens of
commands. Set `interval-ms = 0` if you only want updates from `on-action` (no
polling at all).

Click actions can also open dropdowns. Use `"dropdown:<name>"` for built-in
dropdowns (audio, battery, etc.) or `"dropdown:custom:<name>"` for custom
dropdowns defined in `[dropdowns.custom.<name>]`:

```toml
[[modules.custom]]
id = "scale"
command = "hypr-scale.sh get"
interval-ms = 5000
format = "{{ text }}"
icon-name = "ld-monitor-symbolic"
left-click = "dropdown:custom:scale"
on-action = "hypr-scale.sh get"

[dropdowns.custom.scale]
title = "Display Scale"
icon = "ld-monitor-symbolic"
width = 340

[[dropdowns.custom.scale.sections]]
type = "picker"
list-command = "hypr-scale.sh list --json"
select-command = 'hypr-scale.sh set "$WAYLE_SELECTED"'
```

The `list-command` outputs one item per line — either plain text or JSON:

```json
{"value": "1.5", "label": "1.5x", "subtitle": "140 DPI", "active": true}
```

JSON fields: `value` (selection key), `label`, `subtitle`, `icon`, `active`
(shows checkmark). Plain text lines use the text as both value and label.

The `select-command` receives the chosen value via `$WAYLE_SELECTED`. After
selection the dropdown closes and `on-action` runs to refresh the bar label.

### JSON Reserved Fields

When outputting JSON, these fields have special meaning:

| Field        | Type         | Effect                                     |
| ------------ | ------------ | ------------------------------------------ |
| `text`       | string       | Replaces the `format` result for the label |
| `tooltip`    | string       | Replaces the `tooltip-format` result       |
| `percentage` | number       | 0-100, selects from `icon-names`           |
| `alt`        | string       | Selects from `icon-map`                    |
| `class`      | string/array | Adds CSS classes to the module             |

All other fields are available in `format` and `tooltip-format` templates.

### Full Reference

<details>
<summary>All fields for <code>[[modules.custom]]</code></summary>

#### Core

| Field                 | Type                                     | Default   | Description                                                    |
| --------------------- | ---------------------------------------- | --------- | -------------------------------------------------------------- |
| `id`                  | string                                   | required  | Unique ID, referenced in layout as `custom-<id>`               |
| `command`             | string                                   | none      | Shell command (`sh -c`). JSON auto-detected                    |
| `mode`                | `"poll"` / `"watch"`                     | `"poll"`  | Poll runs on interval, watch streams stdout                    |
| `interval-ms`         | number                                   | `5000`    | Poll interval. `0` = manual only. Ignored in watch mode        |
| `restart-policy`      | `"never"` / `"on-exit"` / `"on-failure"` | `"never"` | Watch mode only                                                |
| `restart-interval-ms` | number                                   | `1000`    | Watch mode restart delay (doubles on rapid failures, caps 30s) |

#### Display

| Field            | Type   | Default          | Description                                               |
| ---------------- | ------ | ---------------- | --------------------------------------------------------- |
| `format`         | string | `"{{ output }}"` | Template for the label. Use `{{ field }}` for JSON fields |
| `tooltip-format` | string | none             | Template for hover tooltip                                |
| `hide-if-empty`  | bool   | `false`          | Hide when output is empty, `"0"`, or `"false"`            |
| `class-format`   | string | none             | Template for dynamic CSS classes (space-separated)        |

#### Icons

| Field        | Type     | Default | Description                                            |
| ------------ | -------- | ------- | ------------------------------------------------------ |
| `icon-name`  | string   | `""`    | Static fallback icon                                   |
| `icon-names` | string[] | none    | Icons indexed by JSON `percentage` (0-100)             |
| `icon-map`   | table    | none    | Icons keyed by JSON `alt`. `"default"` key as fallback |

#### Styling

| Field              | Type   | Default       | Description                             |
| ------------------ | ------ | ------------- | --------------------------------------- |
| `icon-show`        | bool   | `true`        | Show the icon                           |
| `icon-color`       | color  | `"auto"`      | Icon foreground color                   |
| `icon-bg-color`    | color  | `"auto"`      | Icon container background               |
| `label-show`       | bool   | `true`        | Show the text label                     |
| `label-color`      | color  | `"auto"`      | Label text color                        |
| `label-max-length` | number | `0`           | Truncate after N chars (`0` = no limit) |
| `button-bg-color`  | color  | theme default | Button background                       |
| `border-show`      | bool   | `false`       | Show border                             |
| `border-color`     | color  | `"auto"`      | Border color                            |

#### Actions

| Field          | Type   | Default | Description                                                        |
| -------------- | ------ | ------- | ------------------------------------------------------------------ |
| `left-click`   | string | `""`    | Shell command, `"dropdown:<name>"`, or `"dropdown:custom:<name>"`  |
| `right-click`  | string | `""`    | Shell command, `"dropdown:<name>"`, or `"dropdown:custom:<name>"`  |
| `middle-click` | string | `""`    | Shell command, `"dropdown:<name>"`, or `"dropdown:custom:<name>"`  |
| `scroll-up`    | string | `""`    | Shell command or dropdown action (50ms debounce)                   |
| `scroll-down`  | string | `""`    | Shell command or dropdown action (50ms debounce)                   |
| `on-action`    | string | none    | Runs after shell actions or dropdown close, output updates display |

Color values: `"auto"`, hex (`"#ff0000"`), or theme token (`"red"`, `"primary"`,
etc.).

</details>

## Credits

Logo by [@M70v](https://www.instagram.com/m70v.art/).

## License

MIT
