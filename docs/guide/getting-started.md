---
title: Getting started
---

# Getting started

Wayle is a Wayland desktop shell written in Rust with GTK4 and Relm4. It provides a top bar, notification center, on-screen display, wallpaper management and much more. Built-in mechanisms to handle audio, Bluetooth, network, media, battery and power controls.

Settings can be edited in `config.toml`, through the `wayle-settings` GUI, or with the `wayle config` CLI. All three surfaces provided hot reloaded updates, reflected in the shell immediately.

Wayle requires a Wayland compositor that implements the `wlr-layer-shell` protocol. Compositor-specific modules currently target Hyprland; Niri, Sway, and Mango support is in development.

<a href="/wayle-preview.png" target="_blank" rel="noopener">
  <img src="/wayle-preview.png" alt="Wayle desktop shell" style="margin-bottom: 1.5rem;">
</a>

<a href="/wayle-settings-preview.png" target="_blank" rel="noopener">
  <img src="/wayle-settings-preview.png" alt="Wayle settings GUI">
</a>

## Install

- [Arch Linux](/guide/getting-started-arch)
- [Debian / Ubuntu](/guide/getting-started-debian)
- [Fedora](/guide/getting-started-fedora)

<details>
<summary>Other distros</summary>

You'll need these libraries plus their `-dev` / `-devel` headers:

| Library          | Minimum version |
| ---------------- | --------------- |
| GTK              | 4.12            |
| gtk4-layer-shell | 1.0             |
| GtkSourceView    | 5               |
| libpulse         | 8               |
| fftw3            | 3               |
| libpipewire      | 0.3             |
| libudev          | any             |

Plus a C toolchain: `clang`, `cmake`, `pkg-config`, `git`, and a C/C++ compiler.

</details>
