# Custom styles

`~/.config/wayle/styles/index.scss` is an escape hatch for CSS tweaks that aren't exposed through `config.toml` or the settings GUI. The file is created with a starter comment the first time Wayle runs. Anything written into it is layered on top of the built-in stylesheet, and saved changes apply to the running shell automatically.

For things the config already covers (palette, fonts, global scale, rounding, bar layout, module visibility), use the normal config or the settings GUI. This file is for the rest: per-instance module styling, animation behavior, padding and spacing tweaks, hover and focus state changes, custom keyframe animations.

## Per-instance module styling

Two clock modules on the same bar share every config-level styling option. To style them differently, give one of them a CSS class in the layout entry by writing it as a table instead of a bare string:

```toml
# config.toml
[bar.layout]
center = [
  { module = "clock", class = "primary-clock" },
  "clock",
]
```

The class is added to that module's root widget. The selector to target it from `index.scss` is `menubutton.bar-button.primary-clock` (every module is a `menubutton.bar-button`; the custom class is appended).

The robust way to change appearance on that one instance is to override the CSS variables it consumes. The built-in rules read from variables like `--bar-btn-bg`, `--bar-btn-label-color`, `--bar-btn-icon-size`, and `--bar-btn-label-weight`. Redefining one of those inside the per-instance selector replaces it for that widget only:

```scss
// ~/.config/wayle/styles/index.scss
menubutton.bar-button.primary-clock {
  --bar-btn-bg: var(--palette-primary);
  --bar-btn-label-color: var(--palette-bg);
  --bar-btn-label-weight: 700;
}
```

Other modules continue to use the defaults.

The same `class = "..."` field can be applied to any item in `left`, `center`, or `right`, and to items nested inside groups.

## Finding selectors and variables

```sh
wayle panel inspect
```

The GTK Inspector opens against the running shell. The picker shows every widget's CSS classes, its computed style, and the variables in scope. The full set of variables Wayle defines lives in the styles that ship with the project; the inspector is the practical way to discover which ones a given widget reads from.

### Inspecting dropdowns

Dropdowns close as soon as focus leaves the bar, so they vanish the moment the inspector takes focus. To keep a dropdown open long enough to inspect it, disable autohide first. Either toggle **Bar → Dropdown → Dropdown Autohide** in the settings GUI, or set this in `config.toml`:

```toml
[bar]
dropdown-autohide = false
```

Open the dropdown, then run `wayle panel inspect`. Re-enable autohide afterwards.

GTK4's CSS is a subset of web CSS. See [docs.gtk.org/gtk4/css-overview.html](https://docs.gtk.org/gtk4/css-overview.html) for the full reference.

## Sass features

The styles directory is treated as a Sass project. `@use`, `@forward`, `@import`, mixins, variables, math, and color functions all work. Partials (`_name.scss`) can be imported from `index.scss` by basename:

```scss
// ~/.config/wayle/styles/index.scss
@import "modules";
```

```scss
// ~/.config/wayle/styles/_modules.scss
menubutton.bar-button.primary-clock {
  --bar-btn-bg: var(--palette-primary);
}
```

Any `.scss` or `.css` change anywhere in the directory triggers a reload.

## Disable user styles

Empty the contents of `~/.config/wayle/styles/index.scss` to revert to defaults.
