# Wayle

> **⚠️ Work in Progress**: Wayle is under active development. The bar and
> modules that are completed under the [UI Components](#ui-components) section
> are ready to use. This, however, is not a stable environment, things are
> subject to change.

A fast, configurable desktop environment shell for Wayland compositors. Built in
Rust with Relm4 and focused on performance, modularity, and a great user
experience. A successor to HyprPanel without the pain or dependency on Hyprland.

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
- [ ] **Settings Dialog (WIP)**

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

## Building

Install Rust via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repository recursively and build:

```bash
git clone --recursive https://github.com/wayle-rs/wayle
cd wayle
cargo install --path crates/wayle-shell
cargo install --path wayle
```

Once Wayle is installed, you can set up the icons (temporary measure) and start
it via:

```bash
wayle icons setup
wayle panel start
```

## Icons

Wayle uses GTK symbolic icons that support CSS color theming.

To manually manage icons:

```bash
# Install bundled icons (automatic on first launch)
wayle icons setup

# Install additional icons from CDN sources
wayle icons install tabler home settings bell
wayle icons install simple-icons firefox spotify

# See all available sources
wayle icons install --help
```

Icons are installed to `~/.local/share/wayle/icons/` as GTK symbolic icons.

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

You can also open built-in dropdowns from custom modules:

```toml
left-click = "dropdown:audio"
```

### Dropdown Picker

Custom modules can show an inline dropdown with selectable options — useful for
context switchers like kubectl or gcloud. Set `left-click = "dropdown"` (bare,
no colon) and provide commands to list and select items:

```toml
[[modules.custom]]
id = "kube-context"
command = "kubectl config current-context"
interval-ms = 5000
format = "{{ output }}"
icon-name = "ld-layers-symbolic"
label-max-length = 24
label-ellipsize = "middle"
left-click = "dropdown"
dropdown-list-command = "kubectl config get-contexts -o name"
dropdown-select-command = 'kubectl config use-context "$WAYLE_SELECTED"'
```

- **`dropdown-list-command`** runs on click and populates the dropdown. Each
  non-empty line of output becomes a selectable item.
- **`dropdown-select-command`** runs when the user picks an item. The selected
  value is passed via the `$WAYLE_SELECTED` environment variable — always
  quote it (`"$WAYLE_SELECTED"`) to handle values with spaces.
- The **active item** is determined by matching list entries against the current
  `command` output (exact trimmed text match). It gets a checkmark and accent
  styling.
- After selection, the main `command` re-runs immediately so the label updates
  without waiting for the next poll.

Use `label-ellipsize = "middle"` to truncate long context names in the middle
(`"my-long-…-name"`) instead of at the end. Supported values: `end` (default),
`middle`, `start`.

Another example — gcloud configuration switcher:

```toml
[[modules.custom]]
id = "gcloud-config"
command = 'gcloud config configurations list --filter=is_active=true "--format=value(name)"'
interval-ms = 5000
format = "{{ output }}"
icon-name = "ld-cloud-symbolic"
label-max-length = 20
label-ellipsize = "middle"
left-click = "dropdown"
dropdown-list-command = 'gcloud config configurations list "--format=value(name)"'
dropdown-select-command = 'gcloud config configurations activate "$WAYLE_SELECTED"'
```

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
| `label-ellipsize`  | string | `"end"`       | Where to truncate: `"end"`, `"middle"`, `"start"` |
| `button-bg-color`  | color  | theme default | Button background                       |
| `border-show`      | bool   | `false`       | Show border                             |
| `border-color`     | color  | `"auto"`      | Border color                            |

#### Actions

| Field          | Type   | Default | Description                                   |
| -------------- | ------ | ------- | --------------------------------------------- |
| `left-click`   | string | `""`    | Command, `"dropdown"` for inline picker, or `"dropdown:<name>"` |
| `right-click`  | string | `""`    | Command on right click                        |
| `middle-click` | string | `""`    | Command on middle click                       |
| `scroll-up`    | string | `""`    | Command on scroll up (50ms debounce)          |
| `scroll-down`  | string | `""`    | Command on scroll down (50ms debounce)        |
| `on-action`    | string | none    | Runs after any action, output updates display |

#### Dropdown

| Field                    | Type   | Default | Description                                                     |
| ------------------------ | ------ | ------- | --------------------------------------------------------------- |
| `dropdown-list-command`  | string | none    | Command returning newline-separated items for the dropdown      |
| `dropdown-select-command`| string | none    | Command run on selection; `$WAYLE_SELECTED` has the chosen item |

Color values: `"auto"`, hex (`"#ff0000"`), or theme token (`"red"`, `"primary"`,
etc.).

</details>

## Credits

Big thanks to [@M70v](https://www.instagram.com/m70v.art/) for the Wayle logo
contribution! Check out their work at <https://www.instagram.com/m70v.art/>.

## License

MIT
