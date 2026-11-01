---
title: Themes
---

# Themes

A Wayle theme is a TOML file that defines a color palette. Drop one into `~/.config/wayle/themes/` (or `$XDG_CONFIG_HOME/wayle/themes/`) and the settings dialog lists it alongside the built-in themes. If a file in that directory collides with a built-in name, the built-in wins and the file is skipped.

## Adding a theme

The filename without `.toml` becomes the theme name shown in the settings dialog. `tokyo-night.toml` appears as `tokyo-night`.

```toml
# ~/.config/wayle/themes/tokyo-night.toml
bg = "#1a1b26"
surface = "#24283b"
elevated = "#2f334d"

fg = "#c0caf5"
fg_muted = "#9aa5ce"

primary = "#7aa2f7"

red = "#f7768e"
yellow = "#e0af68"
green = "#9ece6a"
blue = "#7aa2f7"
```

Every key takes a hex string. The roles:

- `bg` - darkest base background
- `surface` - card and sidebar background
- `elevated` - raised element background
- `fg` - primary text
- `fg_muted` - secondary text
- `primary` - accent for interactive elements
- `red`, `yellow`, `green`, `blue` - palette accents

Palette values are referenced from elsewhere in the config via [`ColorValue`](/config/types#color-value).

## Apply a theme

Open the settings dialog with `wayle panel settings`. The themes list shows every built-in theme plus every valid `*.toml` file in `~/.config/wayle/themes/`. Pick one; the shell applies it immediately.

## Editor validation

Wayle writes `themes/schema.json` and a `tombi.toml` mapping alongside it, so any editor running the [Tombi](https://tombi-toml.github.io/tombi/) LSP gets completion and validation for theme files out of the box.
