---
title: keyboard-input
outline: [2, 3]
---

# keyboard-input

<div v-pre>

Active keyboard layout indicator.

Add it to your layout with `keyboard-input`:

```toml
[[bar.layout]]
monitor = "*"
right = ["keyboard-input"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `format` | string | `"{{ alias }}"` | Format string for the label. |
| `icon-name` | string | `"ld-keyboard-symbolic"` | Symbolic icon name. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display text label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |
| `layout-alias-map` | map of string | `{}` | Language name mapping. |

::: details More about `format`

#### Placeholders

- `{{ layout }}` - Raw layout name from the compositor (e.g., "English (US)")
- `{{ alias }}` - User-defined alias from `layout-alias-map`, falls back to `{{ layout }}`

#### Examples

- `"{{ layout }}"` - "English (US)"
- `"{{ alias }}"` - "EN" (with alias map configured)

:::

::: details More about `layout-alias-map`

#### Example

```toml
[modules.keyboard-input.layout-alias-map]
"English (US)" = "EN"
"Czech (QWERTY)" = "Czech"
```

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"yellow"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
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
[modules.keyboard-input]
format = "{{ alias }}"
icon-name = "ld-keyboard-symbolic"
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

[modules.keyboard-input.layout-alias-map]
```


</div>
