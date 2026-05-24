---
title: window-title
outline: [2, 3]
---

# window-title

<div v-pre>

Active window title with optional app-icon prefix.

Add it to your layout with `window-title`:

```toml
[[bar.layout]]
monitor = "*"
right = ["window-title"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `format` | string | `"{{ title }}"` | Format string for the label. |
| `icon-name` | string | `"ld-app-window-symbolic"` | Fallback icon when no mapping matches. |
| `icon-mappings` | map of string | `{}` | Icon mappings. Glob patterns to icon names. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `50` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `format`

#### Placeholders

- `{{ title }}` - Window title
- `{{ app }}` - Application name (WM_CLASS on Hyprland)

#### Examples

- `"{{ title }}"` - "README.md - VSCode"
- `"{{ app }}: {{ title }}"` - "firefox: GitHub"

:::

::: details More about `icon-mappings`

Keys are patterns matching the window class (default) or title (when
prefixed with `title:`). Values are icon names from the installed icon
set. User mappings are checked before built-in mappings.

#### Example

```toml
[modules.window-title.icon-mappings]
"*firefox*" = "ld-globe-symbolic"
"org.mozilla.*" = "ld-globe-symbolic"
"title:*YouTube*" = "ld-youtube-symbolic"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"blue"` | Label text color token. |
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
[modules.window-title]
format = "{{ title }}"
icon-name = "ld-app-window-symbolic"
border-show = false
border-color = "blue"
icon-show = true
icon-color = "auto"
icon-bg-color = "blue"
label-show = true
label-color = "blue"
label-max-length = 50
button-bg-color = "bg-surface-elevated"
left-click = ""
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""

[modules.window-title.icon-mappings]
```


</div>
