# Custom icons

Config fields like `icon-name = "tb-home-symbolic"` resolve against the Wayle icon theme installed at `~/.local/share/wayle/icons/`. The `wayle icons` subcommand populates that directory from CDN sources or local SVG files.

## List available sources

```sh
wayle icons sources
```

Five sources ship in-tree. Slug names come directly from the upstream catalogues, linked below.

| Source | Prefix | Contents | Browse |
|---|---|---|---|
| `tabler` | `tb-` | Outline UI icons | [tabler.io/icons](https://tabler.io/icons) |
| `tabler-filled` | `tbf-` | Solid UI icons | [tabler.io/icons](https://tabler.io/icons) |
| `simple-icons` | `si-` | Brand and app logos | [simpleicons.org](https://simpleicons.org) |
| `material` | `md-` | Google Material Design | [fonts.google.com/icons](https://fonts.google.com/icons) |
| `lucide` | `ld-` | Alternative UI icons (Feather fork) | [lucide.dev/icons](https://lucide.dev/icons) |

## Install from a CDN source

```sh
wayle icons install <source> <slug> [<slug>...]
```

Example:

```sh
wayle icons install tabler home settings bell
```

This downloads the three Tabler icons and installs them as `tb-home-symbolic`, `tb-settings-symbolic`, and `tb-bell-symbolic`. The source name determines the prefix; the slug is the icon's name on the upstream site.

Brand logos work the same way:

```sh
wayle icons install simple-icons firefox spotify
```

Produces `si-firefox-symbolic` and `si-spotify-symbolic`.

## Import a local SVG

```sh
wayle icons import <path> [name]
```

For a single file, pass a name. The icon is installed with the `cm-` prefix:

```sh
wayle icons import ~/Downloads/my-logo.svg my-logo
```

Installs as `cm-my-logo-symbolic`.

For a directory, omit the name. Every SVG is imported; files already starting with a known prefix (`tb-`, `tbf-`, `si-`, `md-`, `ld-`, `cm-`) keep their name, and anything else gets `cm-` added.

```sh
wayle icons import ~/exported-icons/
```

Imported SVGs are transformed into GTK symbolic icons so they pick up theme colours.

## List and remove

```sh
wayle icons list
wayle icons list --source tb
```

`--source` filters by prefix. Add `--interactive` for an fzf-backed search if `fzf` is on `$PATH`.

To remove icons, pass their full names:

```sh
wayle icons remove tb-home-symbolic si-firefox-symbolic
```

## Referencing icons in config

Any module field that takes an icon name accepts installed icons directly:

```toml
[modules.clock]
icon-name = "ld-clock-symbolic"

[modules.network]
icon-name = "tb-wifi-symbolic"
```

The value is the full icon name, prefix included. If the icon is missing from the theme, the module falls back to its default.
