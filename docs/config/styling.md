---
title: styling
outline: [2, 3]
---

# styling

<div v-pre>

Theme, palette, and rounding tokens applied shell-wide. Changes recompile the stylesheet.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `scale` | [`ScaleFactor`](/config/types#scale-factor) | `1.01` | Scale multiplier for dropdowns, popovers, and dialogs. |
| `rounding` | [`RoundingLevel`](/config/types#rounding-level) | `"sm"` | Corner rounding for dropdowns, popovers, and dialogs. |
| `theming-monitor` | string | `""` | Monitor whose wallpaper drives color extraction. Empty uses the first available. |

## Theme provider

| Field | Type | Default | Description |
|---|---|---|---|
| `theme-provider` | [`ThemeProvider`](/config/types#theme-provider) | `"wayle"` | Theme provider (wayle, matugen, pywal, wallust). |

## Matugen

| Field | Type | Default | Description |
|---|---|---|---|
| `matugen-scheme` | [`MatugenScheme`](/config/types#matugen-scheme) | `"tonal-spot"` | Matugen color scheme type. |
| `matugen-contrast` | [`SignedNormalizedF64`](/config/types#signed-normalized-f64) | `0` | Matugen contrast level (-1.0 to 1.0). |
| `matugen-source-color` | u8 | `0` | Matugen source color index (0-3). |
| `matugen-light` | bool | `false` | Matugen light mode. |

## Wallust

| Field | Type | Default | Description |
|---|---|---|---|
| `wallust-palette` | [`WallustPalette`](/config/types#wallust-palette) | `"dark16"` | Wallust palette mode. |
| `wallust-saturation` | [`Percentage`](/config/types#percentage) | `0` | Wallust saturation boost (0-100, 0 disables). |
| `wallust-check-contrast` | bool | `true` | Wallust contrast checking against background. |
| `wallust-backend` | [`WallustBackend`](/config/types#wallust-backend) | `"fastresize"` | Wallust image sampling backend. |
| `wallust-colorspace` | [`WallustColorspace`](/config/types#wallust-colorspace) | `"labmixed"` | Wallust color space for dominant color selection. |
| `wallust-apply-globally` | bool | `true` | Apply wallust colors to terminals and external tools. |

## Pywal

| Field | Type | Default | Description |
|---|---|---|---|
| `pywal-saturation` | [`NormalizedF64`](/config/types#normalized-f64) | `0.05` | Pywal saturation adjustment (0.0-1.0). |
| `pywal-contrast` | [`PywalContrast`](/config/types#pywal-contrast) | `3` | Pywal minimum contrast ratio (1.0-21.0). |
| `pywal-light` | bool | `false` | Pywal light mode. |
| `pywal-apply-globally` | bool | `true` | Apply pywal colors to terminals and external tools. |

## Palette

| Field | Type | Default | Description |
|---|---|---|---|
| `palette` | [`PaletteConfig`](/config/types#palette-config) | `{...}` | Active color palette. |

## Default configuration

```toml
[styling]
scale = 1.0099999904632568
rounding = "sm"
theme-provider = "wayle"
theming-monitor = ""
matugen-scheme = "tonal-spot"
matugen-contrast = 0.0
matugen-source-color = 0
matugen-light = false
wallust-palette = "dark16"
wallust-saturation = 0
wallust-check-contrast = true
wallust-backend = "fastresize"
wallust-colorspace = "labmixed"
wallust-apply-globally = true
pywal-saturation = 0.05
pywal-contrast = 3.0
pywal-light = false
pywal-apply-globally = true

[styling.palette]
bg = "#141420"
surface = "#1c1c2c"
elevated = "#262638"
fg = "#d4d6e8"
fg-muted = "#8a8ca4"
primary = "#e0947a"
red = "#e46870"
yellow = "#e0b870"
green = "#68c898"
blue = "#78a0e0"
```


</div>
