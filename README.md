<p align="center">
  <img src="assets/wayle.svg" width="200" alt="Wayle">
</p>

# Wayle

<p align="center">
  <a href="https://github.com/wayle-rs/wayle/actions"><img src="https://img.shields.io/github/actions/workflow/status/wayle-rs/wayle/ci.yml?branch=master&style=for-the-badge" alt="CI"></a>
  <a href="https://github.com/wayle-rs/wayle/blob/master/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue?style=for-the-badge" alt="License"></a>
  <a href="https://wayle.app"><img src="https://img.shields.io/badge/Wiki-wayle.app-7aa2f7?style=for-the-badge" alt="Wiki"></a>
</p>

A Wayland desktop shell with the bar, notifications, OSD, wallpaper, and device controls built in. Written in Rust with GTK4 and Relm4.

Configure it in `config.toml`, through the `wayle-settings` GUI, or with the `wayle config` CLI.

<p align="center">
  <img src="assets/wayle-preview.png" alt="Wayle desktop shell">
</p>

<p align="center">
  <img src="assets/wayle-settings-preview.png" alt="Wayle settings GUI">
</p>

## Documentation

Full guides, reference, and walkthroughs are at **[wayle.app](https://wayle.app)**.

- [Getting started](https://wayle.app/guide/getting-started) - Installation instructions
- [Editing config](https://wayle.app/guide/editing-config) - File layout, live reload, imports, CLI editing
- [Bars and layouts](https://wayle.app/guide/bars-and-layouts) - Per monitor layouts, groups, classes
- [Themes](https://wayle.app/guide/themes) - Color tokens, theme files
- [Custom icons](https://wayle.app/guide/custom-icons) - Installing icons, icon sources
- [Custom modules](https://wayle.app/guide/custom-modules) - Shell-backed bar modules
- [CLI](https://wayle.app/guide/cli) - Every subcommand
- [Config reference](https://wayle.app/config/) - Full config documentation

## Install

Arch Linux binary:

```sh
yay -S wayle-bin
```

<details>
<summary><b>Arch (from source)</b></summary>

Install Rust via [rustup](https://rustup.rs), then the system libraries:

```sh
sudo pacman -S --needed git gtk4 gtk4-layer-shell gtksourceview5 \
  libpulse fftw libpipewire systemd-libs clang base-devel
```

Runtime daemons for the battery, bluetooth, network, power, and audio modules (skip any you don't need):

```sh
sudo pacman -S --needed bluez bluez-utils networkmanager upower \
  power-profiles-daemon pipewire wireplumber pipewire-pulse
sudo systemctl enable --now bluetooth NetworkManager upower power-profiles-daemon
```

</details>

<details>
<summary><b>Debian / Ubuntu</b></summary>

Ubuntu 24.04 LTS does not package `libgtk4-layer-shell-dev`. Use Ubuntu 25.04+ or Debian 13 (trixie).

Install Rust via [rustup](https://rustup.rs), then the system libraries:

```sh
sudo apt install git pkg-config cmake libgtk-4-dev libgtk4-layer-shell-dev \
  libgtksourceview-5-dev libpulse-dev libfftw3-dev libpipewire-0.3-dev \
  libudev-dev clang build-essential
```

Runtime daemons:

```sh
sudo apt install dbus-user-session bluez network-manager \
  upower power-profiles-daemon pipewire-pulse wireplumber
sudo systemctl enable --now bluetooth NetworkManager upower power-profiles-daemon
```

</details>

<details>
<summary><b>Fedora</b></summary>

Requires Fedora 42 or later.

Install Rust via [rustup](https://rustup.rs), then the system libraries:

```sh
sudo dnf install git cmake pkgconf-pkg-config gtk4-devel gtk4-layer-shell-devel \
  gtksourceview5-devel pulseaudio-libs-devel fftw-devel pipewire-devel \
  systemd-devel clang gcc
```

Fedora Workstation already ships the runtime daemons. Minimal and Server installs need:

```sh
sudo dnf install pipewire-pulseaudio wireplumber NetworkManager bluez upower \
  power-profiles-daemon
sudo systemctl enable --now bluetooth NetworkManager upower power-profiles-daemon
```

</details>

### Build and launch:

```sh
git clone https://github.com/wayle-rs/wayle
cd wayle
cargo install --path wayle
cargo install --path crates/wayle-settings
wayle icons setup
wayle panel start
```

On a different distro? See [wayle.app/guide/getting-started](https://wayle.app/guide/getting-started) for the library-version reference.

## Configuration

The config file is at `~/.config/wayle/config.toml`. Changes reload on save:

```toml
[bar]
location = "top"
scale = 1.25

[[bar.layout]]
monitor = "*"
left = ["dashboard"]
center = ["clock"]
right = ["volume", "network", "bluetooth", "battery"]

[modules.clock]
format = "%H:%M"
```

Every field is documented at [wayle.app/config](https://wayle.app/config/).

## Requirements

A Wayland compositor that implements the `wlr-layer-shell` protocol. Compositor-specific modules currently target Hyprland; Niri, Sway, and Mango support is in development.

## Credits

Logo by [@M70v](https://www.instagram.com/m70v.art/).

## License

MIT
