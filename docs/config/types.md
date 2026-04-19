---
title: Types
outline: [2, 3]
---

<div v-pre>

# Types

Named types referenced across the config. Every field in [`/config/`](/config/) that shows a type like `Color` or `ClickAction` links here.

## ActiveIndicator {#active-indicator}

Visual indicator style for the active workspace.

| Value | Meaning |
|---|---|
| `"background"` | Entire button gets a colored background. |
| `"underline"` | Small colored bar under the workspace button. |

## AppIconSource {#app-icon-source}

Icon source for app volume entries in the dropdown.

| Value | Meaning |
|---|---|
| `"mapped"` | Wayle's curated symbolic icons matched by app name. |
| `"native"` | Native application icons reported by PulseAudio. |

## BarButtonVariant {#bar-button-variant}

Visual style variants for bar buttons.

| Value | Meaning |
|---|---|
| `"basic"` | Icon + label, minimal background. |
| `"block-prefix"` | Icon in colored pill container that blends into button edge. |
| `"icon-square"` | Button background with colored icon container inside. |

## BarCount {#bar-count}

Frequency bar count clamped to 1-256 (mirrors `wayle_cava::BarCount`).

Number in `[1, 256]`.

Serialises as `uint16`.

## BarGroup {#bar-group}

Named group of modules. The name becomes a CSS ID selector.

| Field | Description |
|---|---|
| `name` | Unique name for CSS targeting (becomes `#name` selector). |
| `modules` | Modules contained in this group. |

## BarItem {#bar-item}

One entry in a bar layout section (`left`, `center`, or `right`).

Three shapes are accepted, all interchangeable in the same array:

- A plain module name: `"clock"`
- A module with a CSS class for per-instance styling: `{ module = "clock", class = "primary" }`
- A named group that wraps several modules in a shared container, addressable by CSS ID

### Examples

```toml
[[bar.layout]]
monitor = "*"

# Plain module
left = ["dashboard"]

# Mix of plain and classed modules on the same side
center = ["clock", { module = "clock", class = "secondary" }]

# Named group (renders inside a GTK container with CSS ID `#status`)
right = [{ name = "status", modules = ["battery", "network", "volume"] }]

# Groups can hold classed modules too
[[bar.layout]]
monitor = "DP-2"
left = [{ name = "clocks", modules = [
  { module = "clock", class = "local" },
  { module = "world-clock", class = "remote" }
]}]
```

## BarLayout {#bar-layout}

Layout configuration for a bar on a specific monitor.

### Examples

```toml
# Single modules
[[bar.layout]]
monitor = "*"
left = ["dashboard"]
center = ["clock"]
right = ["systray"]

# Module with custom CSS class for per-instance styling
[[bar.layout]]
monitor = "DP-1"
left = [{ module = "clock", class = "primary-clock" }, "clock"]
center = ["media"]

# Grouped modules (share a visual container, CSS-targetable by name)
[[bar.layout]]
monitor = "DP-2"
left = [{ name = "status", modules = ["battery", "network"] }]

# Groups can also contain classed modules
[[bar.layout]]
monitor = "DP-3"
left = [{ name = "clocks", modules = [
  { module = "clock", class = "local" },
  { module = "world-clock", class = "remote" }
]}]

# Inherit from another layout
[[bar.layout]]
monitor = "*"
left = ["dashboard"]
center = ["clock"]
right = ["systray"]

[[bar.layout]]
monitor = "HDMI-1"
extends = "*"
right = ["volume", "systray"]  # Override just this section

# Hide bar on a specific monitor
[[bar.layout]]
monitor = "HDMI-2"
show = false
```

| Field | Description |
|---|---|
| `monitor` | Monitor connector name (e.g., `"DP-1"`) or `"*"` for all monitors. |
| `extends` | Inherit from another layout by its monitor value (e.g., `"*"`). |
| `show` | Whether the bar is visible on this monitor. |
| `left` | Modules in the left section. |
| `center` | Modules in the center section. |
| `right` | Modules in the right section. |

## BarModule {#bar-module}

Bar module name. Built-in modules or custom modules with a `custom-<id>` pattern.

One of: `"battery"`, `"bluetooth"`, `"cava"`, `"clock"`, `"cpu"`, `"dashboard"`, `"hyprland-workspaces"`, `"hyprsunset"`, `"idle-inhibit"`, `"keybind-mode"`, `"keyboard-input"`, `"media"`, `"microphone"`, `"netstat"`, `"network"`, `"notifications"`, `"power"`, `"ram"`, `"separator"`, `"storage"`, `"systray"`, `"updates"`, `"volume"`, `"weather"`, `"window-title"`, `"world-clock"`.

String matching `^custom-[a-z0-9-]+$`.

## BorderLocation {#border-location}

Border placement for bar buttons.

| Value | Meaning |
|---|---|
| `"none"` | No border. |
| `"top"` | Border on top edge only. |
| `"bottom"` | Border on bottom edge only. |
| `"left"` | Border on left edge only. |
| `"right"` | Border on right edge only. |
| `"all"` | Border on all edges. |

## CavaDirection {#cava-direction}

Bar growth direction relative to the bar's attached screen edge.

| Value | Meaning |
|---|---|
| `"normal"` | Bars grow away from the attached edge. |
| `"reverse"` | Bars grow toward the attached edge. |
| `"mirror"` | Bars grow symmetrically from center. |

## CavaInput {#cava-input}

Audio capture backend.

| Value | Meaning |
|---|---|
| `"pipe-wire"` | PipeWire multimedia server. |
| `"pulse"` | PulseAudio sound server. |
| `"alsa"` | Advanced Linux Sound Architecture. |
| `"jack"` | JACK Audio Connection Kit. |
| `"fifo"` | Named pipe (FIFO) input. |
| `"port-audio"` | PortAudio cross-platform library. |
| `"sndio"` | sndio audio subsystem (BSD). |
| `"oss"` | Open Sound System (legacy). |
| `"shmem"` | Shared memory input. |
| `"winscap"` | Windows audio capture (WASAPI). |

## CavaStyle {#cava-style}

Visualization rendering style.

| Value | Meaning |
|---|---|
| `"bars"` | Rectangular frequency bars. |
| `"wave"` | Smooth curve connecting bar peaks. |
| `"peaks"` | Bars with floating peak indicators that decay over time. |

## ClassedModule {#classed-module}

A module with an associated CSS class for custom styling.

| Field | Description |
|---|---|
| `module` | The module type. |
| `class` | CSS class added to the module's GTK widget. |

## ClickAction {#click-action}

String.

## ColorValue {#color-value}

CSS token, hex color (#rgb, #rgba, #rrggbb, or #rrggbbaa), 'transparent', or 'auto'

| Value | Meaning |
|---|---|
| `"bg-base"` | `--bg-base` - Application background. |
| `"bg-surface"` | `--bg-surface` - Elevated surfaces. |
| `"bg-surface-elevated"` | `--bg-surface-elevated` - Subtle elevation from surface (buttons on surface). |
| `"bg-elevated"` | `--bg-elevated` - Higher elevation surfaces. |
| `"bg-overlay"` | `--bg-overlay` - Popovers, dialogs. |
| `"bg-hover"` | `--bg-hover` - Hover state background. |
| `"bg-active"` | `--bg-active` - Active/pressed state background. |
| `"bg-selected"` | `--bg-selected` - Selected item background. |
| `"fg-default"` | `--fg-default` - Primary text color. |
| `"fg-muted"` | `--fg-muted` - Secondary text color. |
| `"fg-subtle"` | `--fg-subtle` - Tertiary/hint text color. |
| `"fg-on-accent"` | `--fg-on-accent` - Text color on accent backgrounds. |
| `"accent"` | `--accent` - Primary accent color. |
| `"accent-subtle"` | `--accent-subtle` - Subtle accent background. |
| `"accent-hover"` | `--accent-hover` - Accent hover state. |
| `"status-error"` | `--status-error` - Error state color. |
| `"status-warning"` | `--status-warning` - Warning state color. |
| `"status-success"` | `--status-success` - Success state color. |
| `"status-info"` | `--status-info` - Info state color. |
| `"status-error-subtle"` | `--status-error-subtle` - Subtle error background. |
| `"status-warning-subtle"` | `--status-warning-subtle` - Subtle warning background. |
| `"status-success-subtle"` | `--status-success-subtle` - Subtle success background. |
| `"status-info-subtle"` | `--status-info-subtle` - Subtle info background. |
| `"status-error-hover"` | `--status-error-hover` - Error hover state. |
| `"red"` | `--red` - Red color for stylistic/decorative use. |
| `"yellow"` | `--yellow` - Yellow color for stylistic/decorative use. |
| `"green"` | `--green` - Green color for stylistic/decorative use. |
| `"blue"` | `--blue` - Blue color for stylistic/decorative use. |
| `"border-subtle"` | `--border-subtle` - Subtle border color. |
| `"border-default"` | `--border-default` - Default border color. |
| `"border-strong"` | `--border-strong` - Strong border color. |
| `"border-accent"` | `--border-accent` - Accent-colored border. |
| `"border-error"` | `--border-error` - Error state border. |

One of: `"transparent"`, `"auto"`.

String matching `^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$`.

## CyclingInterval {#cycling-interval}

Cycling interval in minutes, minimum 1.

Number `>= 1`.

Serialises as `uint64`.

## CyclingMode {#cycling-mode}

Wallpaper cycling order.

| Value | Meaning |
|---|---|
| `"sequential"` | Alphabetical order. |
| `"shuffle"` | Random order. |

## DisplayMode {#display-mode}

What identifies a workspace in the UI.

| Value | Meaning |
|---|---|
| `"label"` | Show workspace number or name. |
| `"icon"` | Show icon from `workspace-map` (falls back to label if unmapped). |
| `"none"` | Show nothing - only app icons visible. |

## ExecutionMode {#execution-mode}

Execution mode for custom module commands.

| Value | Meaning |
|---|---|
| `"poll"` | Run command at regular intervals defined by `interval-ms`.

Best for commands that complete quickly and return current state
(e.g., reading a file, querying system status). |
| `"watch"` | Spawn long-running process and update display on each stdout line.

Best for event-driven updates without polling overhead
(e.g., `pactl subscribe`, `inotifywait`, `tail -f`).
Configure `restart-policy` to control restarts after exit. |

## FitMode {#fit-mode}

Image scaling mode.

| Value | Meaning |
|---|---|
| `"fill"` | Scale to cover entire display, cropping excess. |
| `"fit"` | Scale to fit within display, letterboxing if needed. |
| `"center"` | Display at original size, centered. |
| `"stretch"` | Stretch to exactly fill, ignoring aspect ratio. |

## FontWeightClass {#font-weight-class}

Font weight class for typography.

Maps to CSS classes like `.weight-normal`, `.weight-bold`, etc.
Uses the existing `--weight-*` tokens defined in SCSS.

| Value | Meaning |
|---|---|
| `"normal"` | Normal weight (--weight-normal: 400). |
| `"medium"` | Medium weight (--weight-medium: 500). |
| `"semibold"` | Semi-bold weight (--weight-semibold: 600). |
| `"bold"` | Bold weight (--weight-bold: 700). |

## Framerate {#framerate}

Visualization framerate clamped to 1-360 fps (mirrors `wayle_cava::Framerate`).

Number in `[1, 360]`.

Serialises as `uint32`.

## FrequencyHz {#frequency-hz}

Frequency value in Hz, minimum 1 Hz.

Cross-field constraints (high_cutoff > low_cutoff, samplerate/2 > high_cutoff)
are validated at the service builder.

Number `>= 1`.

Serialises as `uint32`.

## HexColor {#hex-color}

GTK4 CSS hex color (#rgb, #rgba, #rrggbb, or #rrggbbaa)

String matching `^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$`.

## IconPosition {#icon-position}

Icon position within bar buttons.

| Value | Meaning |
|---|---|
| `"start"` | Icon before label (left for horizontal, top for vertical bars). |
| `"end"` | Icon after label (right for horizontal, bottom for vertical bars). |

## IconSource {#icon-source}

Source for resolving notification icons.

| Value | Meaning |
|---|---|
| `"automatic"` | Use per-notification images when provided, otherwise Wayle's mapped icon. |
| `"mapped"` | Always use Wayle's mapped icons regardless of what the app provides. |
| `"application"` | Use the full application icon chain, falling back to mapped if unavailable. |

## Location {#location}

Bar position on screen.

| Value | Meaning |
|---|---|
| `"top"` | Top edge of the screen. |
| `"bottom"` | Bottom edge of the screen. |
| `"left"` | Left edge of the screen. |
| `"right"` | Right edge of the screen. |

## MatugenScheme {#matugen-scheme}

Matugen color scheme type.

| Value | Meaning |
|---|---|
| `"content"` | Adapts to image content. |
| `"expressive"` | Bold, dramatic palette. |
| `"fidelity"` | Stays close to source colors. |
| `"fruit-salad"` | Playful multi-color palette. |
| `"monochrome"` | Single-hue grayscale palette. |
| `"neutral"` | Muted, understated palette. |
| `"rainbow"` | Broad hue spread. |
| `"tonal-spot"` | Balanced Material You default. |
| `"vibrant"` | High-saturation palette. |

## MediaIconType {#media-icon-type}

Icon display mode for the media module.

| Value | Meaning |
|---|---|
| `"default"` | Static icon from icon-name field. |
| `"application"` | Dynamic icon from media player's desktop entry, falling back to icon-name. |
| `"spinning-disc"` | Spinning disc icon that animates during playback. Uses slightly more CPU. |
| `"application-mapped"` | Maps player to icon via glob patterns, with built-in mappings for common players. |

## ModuleRef {#module-ref}

Reference to a module, optionally with a custom CSS class.

### Examples

```toml
# Plain module (just the name)
left = ["clock"]

# Module with custom CSS class
left = [{ module = "clock", class = "primary-clock" }]
```

## MonitorWallpaperConfig {#monitor-wallpaper-config}

Per-monitor wallpaper configuration.

| Field | Description |
|---|---|
| `name` | Monitor name (e.g., "HDMI-1", "DP-1"). |
| `fit-mode` | Image scaling mode for this monitor. |
| `wallpaper` | Wallpaper image path for this monitor. |

## NormalizedF64 {#normalized-f64}

Floating-point value clamped to 0.0-1.0.

Number in `[0.0, 1.0]`.

Serialises as `double`.

## Numbering {#numbering}

How workspace numbers are displayed.

| Value | Meaning |
|---|---|
| `"absolute"` | Show actual Hyprland workspace IDs (1, 2, 3, 4, 5, 6...). |
| `"relative"` | Show numbers relative to monitor's starting workspace.

If monitor has workspaces 4, 5, 6 assigned, they display as 1, 2, 3.
Useful when keybinds use per-monitor numbering (Shift+1 for ws 4, etc.). |

## OsdMonitor {#osd-monitor}

"primary" or a monitor connector name (e.g. "DP-1")

String.

## OsdPosition {#osd-position}

Screen anchor for the OSD overlay.

| Value | Meaning |
|---|---|
| `"top-left"` | Top-left corner. |
| `"top"` | Top-center edge. |
| `"top-right"` | Top-right corner. |
| `"right"` | Right-center edge. |
| `"bottom-right"` | Bottom-right corner. |
| `"bottom"` | Bottom-center edge. |
| `"bottom-left"` | Bottom-left corner. |
| `"left"` | Left-center edge. |

## PaletteConfig {#palette-config}

Color palette configuration for the active theme.

| Field | Description |
|---|---|
| `bg` | Base background color (darkest). |
| `surface` | Card and sidebar background. |
| `elevated` | Raised element background. |
| `fg` | Primary text color. |
| `fg-muted` | Secondary text color. |
| `primary` | Accent color for interactive elements. |
| `red` | Red semantic color. |
| `yellow` | Yellow semantic color. |
| `green` | Green semantic color. |
| `blue` | Blue semantic color. |

## Percentage {#percentage}

Percentage value clamped to 0-100.

Number in `[0, 100]`.

Serialises as `uint8`.

## PopupCloseBehavior {#popup-close-behavior}

Behavior when the close button is clicked on a popup card.

| Value | Meaning |
|---|---|
| `"dismiss"` | Hide the popup; notification stays in history. |
| `"remove"` | Remove the notification entirely. |

## PopupMonitor {#popup-monitor}

"primary" or a monitor connector name (e.g. "DP-1")

String.

## PopupPosition {#popup-position}

Screen position for notification popups.

| Value | Meaning |
|---|---|
| `"top-left"` | Top-left corner. |
| `"top-center"` | Top-center edge. |
| `"top-right"` | Top-right corner. |
| `"bottom-left"` | Bottom-left corner. |
| `"bottom-center"` | Bottom-center edge. |
| `"bottom-right"` | Bottom-right corner. |
| `"center-left"` | Center-left edge. |
| `"center-right"` | Center-right edge. |

## PywalContrast {#pywal-contrast}

Pywal contrast ratio clamped to 1.0-21.0 (WCAG range).

Number in `[1.0, 21.0]`.

Serialises as `double`.

## RestartDelay {#restart-delay}

Restart delay in milliseconds, clamped to >= 1.

Number `>= 1`.

Serialises as `uint64`.

## RestartPolicy {#restart-policy}

Restart behavior for watch-mode custom modules.

| Value | Meaning |
|---|---|
| `"never"` | Never restart after exit. |
| `"on-exit"` | Restart after any exit code (success or failure). |
| `"on-failure"` | Restart only after non-zero exit codes or signal termination. |

## RoundingLevel {#rounding-level}

Global rounding preference for UI components.

| Value | Meaning |
|---|---|
| `"none"` | Sharp corners (no rounding). |
| `"sm"` | Subtle rounding. |
| `"md"` | Moderate rounding (default). |
| `"lg"` | Pronounced rounding. |
| `"full"` | Pill shape (fully rounded ends). |

## ScaleFactor {#scale-factor}

Scale multiplier clamped to 0.25-3.0.

Number in `[0.25, 3.0]`.

Serialises as `float`.

## ShadowPreset {#shadow-preset}

Shadow style for the bar.

| Value | Meaning |
|---|---|
| `"none"` | No shadow. |
| `"drop"` | Directional shadow opposite the anchor edge. |
| `"floating"` | All-around shadow. |

## SignedNormalizedF64 {#signed-normalized-f64}

Floating-point value clamped to -1.0 to 1.0.

Number in `[-1.0, 1.0]`.

Serialises as `double`.

## Spacing {#spacing}

Non-negative spacing value (clamped at 0).

Number `>= 0.0`.

Serialises as `float`.

## StackingOrder {#stacking-order}

Order in which popups are stacked on screen.

| Value | Meaning |
|---|---|
| `"newest-first"` | Newest notifications appear closest to the configured position. |
| `"oldest-first"` | Oldest notifications appear closest to the configured position. |

## TemperatureUnit {#temperature-unit}

Temperature unit for display.

| Value | Meaning |
|---|---|
| `"metric"` | Celsius (metric). |
| `"imperial"` | Fahrenheit (imperial). |

## ThemeProvider {#theme-provider}

Source of color palette values.

Dynamic providers (Matugen, Pywal, Wallust) inject palette tokens at runtime.

| Value | Meaning |
|---|---|
| `"wayle"` | Static theming using Wayle's built-in palettes. |
| `"matugen"` | Dynamic theming via Matugen. |
| `"pywal"` | Dynamic theming via Pywal. |
| `"wallust"` | Dynamic theming via Wallust. |

## ThresholdEntry {#threshold-entry}

A threshold entry that maps a numeric value range to color overrides.

At least one of `above` or `below` must be set. When both are set,
both conditions must be satisfied (AND logic).

### TOML Example

```toml
[[modules.cpu.thresholds]]
above = 70
icon-color = "status-warning"
label-color = "status-warning"

[[modules.cpu.thresholds]]
above = 90
icon-color = "status-error"
label-color = "status-error"
```

| Field | Description |
|---|---|
| `above` | Activate when metric value >= this threshold. |
| `below` | Activate when metric value <= this threshold. |
| `icon-color` | Override icon color when threshold is active. |
| `label-color` | Override label color when threshold is active. |
| `icon-bg-color` | Override icon background color when threshold is active. |
| `button-bg-color` | Override button background color when threshold is active. |
| `border-color` | Override border color when threshold is active. |

## TimeFormat {#time-format}

Time display format.

| Value | Meaning |
|---|---|
| `"12h"` | 12-hour format with AM/PM (e.g., "6:30 AM"). |
| `"24h"` | 24-hour format (e.g., "06:30"). |

## TransitionDuration {#transition-duration}

Transition duration in seconds, clamped to >= 0.

Number `>= 0.0`.

Serialises as `float`.

## TransitionFps {#transition-fps}

Transition frame rate clamped to 1-360 fps.

Number in `[1, 360]`.

Serialises as `uint32`.

## TransitionType {#transition-type}

Transition animation type.

| Value | Meaning |
|---|---|
| `"none"` | Instant change with no animation. |
| `"simple"` | Basic crossfade. |
| `"fade"` | Fade with bezier-controlled easing. |
| `"left"` | Wipe from left edge to right. |
| `"right"` | Wipe from right edge to left. |
| `"top"` | Wipe from top edge to bottom. |
| `"bottom"` | Wipe from bottom edge to top. |
| `"wipe"` | Wipe at configurable angle. |
| `"wave"` | Wavy wipe effect. |
| `"grow"` | Growing circle from a position. |
| `"center"` | Growing circle from center. |
| `"outer"` | Shrinking circle from edges inward. |
| `"any"` | Growing circle from random position. |
| `"random"` | Randomly selects from all transition types. |

## TrayItemOverride {#tray-item-override}

Custom icon and color override for tray items matching a pattern.

| Field | Description |
|---|---|
| `name` | Glob pattern to match against item ID or title. |
| `icon` | Custom icon name (symbolic icon). |
| `color` | Custom icon color. |

## UrgencyBarThreshold {#urgency-bar-threshold}

Minimum urgency level that shows a colored urgency bar on popup cards.

All urgency levels at or above the threshold display the bar.
For example, `Normal` shows bars on both normal and critical popups.

| Value | Meaning |
|---|---|
| `"low"` | Show urgency bars on all popups. |
| `"normal"` | Show urgency bars on normal and critical popups. |
| `"critical"` | Show urgency bars on critical popups only. |
| `"none"` | Never show urgency bars. |

## UrgentMode {#urgent-mode}

Where the urgent pulse animation is applied.

| Value | Meaning |
|---|---|
| `"workspace"` | Pulse the entire workspace. |
| `"application"` | Pulse only the app icon(s) belonging to the urgent window.

Falls back to `workspace` when app icons are disabled. |

## WallustBackend {#wallust-backend}

Wallust image sampling backend.

| Value | Meaning |
|---|---|
| `"full"` | Reads every pixel. |
| `"resized"` | Resizes image before sampling. |
| `"wal"` | Uses ImageMagick convert (pywal method). |
| `"thumb"` | Fixed 512x512 thumbnail. |
| `"fastresize"` | SIMD-accelerated resize. |
| `"kmeans"` | K-means clustering. |

## WallustColorspace {#wallust-colorspace}

Wallust color space for dominant color selection.

| Value | Meaning |
|---|---|
| `"lab"` | CIELAB perceptual color space. |
| `"labmixed"` | LAB with mixing for sparse images. |
| `"lch"` | Cylindrical LAB (hue/chroma/lightness). |
| `"lchmixed"` | LCH with mixing. |
| `"lchansi"` | LCH mapped to ANSI color ordering. |

## WallustPalette {#wallust-palette}

Wallust palette mode.

| Value | Meaning |
|---|---|
| `"dark16"` | 8 dark colors with 16-color trick. |
| `"dark"` | 8 dark colors, dark background and light contrast. |
| `"darkcomp"` | Dark with complementary counterparts. |
| `"darkcomp16"` | Dark complementary with 16-color trick. |
| `"harddark"` | Dark with hard hue colors. |
| `"harddark16"` | Hard dark with 16-color trick. |
| `"harddarkcomp"` | Hard dark complementary variant. |
| `"harddarkcomp16"` | Hard dark complementary with 16-color trick. |
| `"light"` | Light background, dark foreground. |
| `"light16"` | Light with 16-color trick. |
| `"lightcomp"` | Light with complementary colors. |
| `"lightcomp16"` | Light complementary with 16-color trick. |
| `"softdark"` | Lightest colors with dark background. |
| `"softdark16"` | Soft dark with 16-color trick. |
| `"softdarkcomp"` | Soft dark complementary variant. |
| `"softdarkcomp16"` | Soft dark complementary with 16-color trick. |
| `"softlight"` | Light with soft pastel colors. |
| `"softlight16"` | Soft light with 16-color trick. |
| `"softlightcomp"` | Soft light with complementary colors. |
| `"softlightcomp16"` | Soft light complementary with 16-color trick. |
| `"ansidark"` | ANSI-ordered dark palette for LS_COLORS. |
| `"ansidark16"` | ANSI dark with 16-color trick. |

## WeatherProvider {#weather-provider}

Weather data provider selection.

| Value | Meaning |
|---|---|
| `"open-meteo"` | Open-Meteo (no API key required). |
| `"visual-crossing"` | Visual Crossing (requires API key). |
| `"weather-api"` | WeatherAPI.com (requires API key). |

## WorkspaceMap {#workspace-map}

Per-workspace icon and color overrides, keyed by workspace ID.

TOML table keys are always strings, so `"1"` parses into the workspace
with ID `1`. Negative IDs refer to Hyprland's special workspaces. Keys
that don't appear in the map fall back to the default behaviour set by
[`HyprlandWorkspacesConfig::display_mode`].

### Examples

```toml
[modules.hyprland-workspaces.workspace-map]
# Whole entry on one line with an inline table
1 = { icon = "ld-globe-symbolic", color = "#4a90d9" }
2 = { icon = "ld-terminal-symbolic" }
3 = { icon = "ld-code-symbolic", color = "accent" }

# Or spread the entry across its own subtable
[modules.hyprland-workspaces.workspace-map.4]
icon = "ld-message-square-symbolic"
color = "status-success"

# Negative IDs target Hyprland special workspaces
[modules.hyprland-workspaces.workspace-map.-99]
icon = "ld-scratch-symbolic"
```

## WorkspaceStyle {#workspace-style}

Per-workspace styling override.

| Field | Description |
|---|---|
| `icon` | Custom icon for this workspace. |
| `color` | Custom background color for this workspace when active. |


</div>
