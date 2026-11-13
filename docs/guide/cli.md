# CLI

Every subcommand takes `--help`.

Panel lifecycle:

```sh
wayle panel start
wayle panel restart
wayle panel settings
```

Read and edit config values from the command line:

```sh
wayle config get bar.scale
wayle config set bar.scale 1.25
wayle config reset bar.scale
```

Audio, media, and idle controls:

```sh
wayle audio output-volume +5
wayle media play-pause
wayle idle toggle
```

Show an OSD on demand, even when the value hasn't changed — useful on a
keybind. Run `wayle osd devices` to see which device names are accepted:

```sh
wayle osd speaker
wayle osd mic
wayle osd brightness intel_backlight
```

Pair these with the `osd.auto-*` settings to stop an OSD appearing on every
change. For example, when another program adjusts the backlight continuously:

```sh
wayle config set osd.auto-brightness false
```

The `auto-*` settings only control whether a change *opens* the overlay. While
it is open it keeps tracking the device's value, so an OSD you brought up by
hand still shows live readings, and that tracking never restarts the dismiss
timer. With the matching `auto-*` key left on, though, a change to the default
device also re-triggers the automatic display, which does restart it.

Shell completions for bash, fish, and zsh:

```sh
wayle completions fish > ~/.config/fish/completions/wayle.fish
```
