---
title: media
outline: [2, 3]
---

# media

<div v-pre>

Now-playing title and playback controls for the active MPRIS player.

Add it to your layout with `media`:

```toml
[[bar.layout]]
monitor = "*"
right = ["media"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `icon-type` | [`MediaIconType`](/config/types#media-icon-type) | `"application-mapped"` | Icon display mode. |
| `player-icons` | map of string | `{}` | Custom player-to-icon mappings for application-mapped mode. |
| `players-ignored` | array of string | `[]` | Player bus name patterns to exclude from discovery. Requires a restart to take effect. |
| `player-priority` | array of string | `[]` | Preferred player priority order as glob patterns matching bus names. |
| `format` | string | `"{{ title }} - {{ artist }}"` | Format string for the label. |
| `icon-name` | string | `"ld-music-symbolic"` | Symbolic icon name for default mode. |
| `spinning-disc-icon` | string | `"ld-disc-3-symbolic"` | Icon shown for spinning-disc mode. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `35` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `player-icons`

Keys are glob patterns matching MPRIS bus names, values are icon names
from the installed icon set. These override built-in mappings when
matched.

#### Example

```toml
[modules.media.player-icons]
"*spotify*" = "si-spotify-symbolic"
"*firefox*" = "ld-globe-symbolic"
"*.mpv" = "ld-play-circle-symbolic"
```

:::

::: details More about `players-ignored`

#### Example

```toml
[modules.media]
players-ignored = ["*chromium*", "*discord*"]
```

:::

::: details More about `player-priority`

When no player is manually selected, this determines which player
becomes active. Patterns are checked in order; first match wins.
If no pattern matches, the first playing player is selected.

#### Example

```toml
[modules.media]
player-priority = ["*spotify*", "*firefox*"]
```

:::

::: details More about `format`

#### Placeholders

- `{{ title }}` - Track title
- `{{ artist }}` - Artist name(s)
- `{{ album }}` - Album name
- `{{ status }}` - Playback status text (Playing, Paused, Stopped)
- `{{ status_icon }}` - Playback status icon character

#### Examples

- `"{{ title }} - {{ artist }}"` - "Bohemian Rhapsody - Queen"
- `"{{ status_icon }} {{ title }}"` - "▶ Bohemian Rhapsody"
- `"{{ artist }}: {{ title }} ({{ album }})"` - "Queen: Bohemian Rhapsody (A Night at the Opera)"

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:media"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.media]
icon-type = "application-mapped"
players-ignored = []
player-priority = []
format = "{{ title }} - {{ artist }}"
icon-name = "ld-music-symbolic"
spinning-disc-icon = "ld-disc-3-symbolic"
border-show = false
border-color = "blue"
icon-show = true
icon-color = "auto"
icon-bg-color = "blue"
label-show = true
label-color = "blue"
label-max-length = 35
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:media"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""

[modules.media.player-icons]
```


</div>
