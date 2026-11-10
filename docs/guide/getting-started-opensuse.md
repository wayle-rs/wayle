---
title: Getting started on OpenSUSE
---

# Getting started on OpenSUSE

Requires OpenSUSE [Tumbleweed](https://get.opensuse.org/tumbleweed/) or it's monthly release [Slowroll](https://en.opensuse.org/Portal:Slowroll).

`gtk4-layer-shell-devel` is only available by default in Tumbleweed and even if you add [X11:Wayland](https://build.opensuse.org/project/show/X11:Wayland) in Leap 16, `niri`/`hyprland` have unresolvable dependencies.

## Install dependencies

Install Rust via [rustup](https://rustup.rs), then the system libraries:

```sh
sudo zypper install git cmake pkgconf-pkg-config gtk4-devel gtk4-layer-shell-devel \
  gtksourceview5-devel libpulse-devel fftw3-devel pipewire-devel \
  systemd-devel clang gcc
```

If you want the battery, bluetooth, network, power, or audio modules, install their daemons (skip any you don't need):

```sh
sudo zypper install pipewire-pulseaudio wireplumber NetworkManager bluez upower \
  power-profiles-daemon
sudo systemctl enable --now bluetooth NetworkManager upower power-profiles-daemon
systemctl reboot
```

## Build

```sh
git clone https://github.com/wayle-rs/wayle
cd wayle
cargo install --path wayle
cargo install --path crates/wayle-settings
```

## Icon assets

Wayle ships icons as source files that get copied into your user data directory on first setup. Run this from the cloned repo, **before** deleting it:

```sh
wayle icons setup
```

## Run

Start the panel in the background:

```sh
wayle panel start
```

Other lifecycle commands: `wayle panel status`, `wayle panel restart`, `wayle panel stop`.

For debugging, run the shell in the foreground so logs print to the terminal:

```sh
wayle shell
```

## Settings GUI

```sh
wayle panel settings
```

This launches `wayle-settings`, which edits the same config the shell reads. Changes apply live. Anything the GUI doesn't cover can still be edited by hand in `config.toml`.

## Configuration

Wayle reads `$XDG_CONFIG_HOME/wayle/config.toml`, falling back to `~/.config/wayle/config.toml`. On first run, if no config exists, Wayle writes a default one. A JSON schema is written to `~/.config/wayle/schema.json` at startup, which editors with a TOML LSP can use for validation and completion.
