# Wayle

> **⚠️ Work in Progress**: Wayle is under active development. The bar and
> modules that are completed under the [UI Components](#ui-components) section
> are ready to use. This, however, is not a stable environment, things are
> subject to change.

A fast, configurable desktop environment shell for Wayland compositors. Built in
Rust with Relm4 and focused on performance, modularity, and a great user
experience. A successor to HyprPanel without the pain or dependency on Hyprland.

## Progress

### Core Infrastructure

- [x] **Configuration System** - Reactive TOML config with schema validation
- [x] **CLI Interface** - Complete command-line management interface
- [x] **Documentation Generator** - Auto-generated config docs from schemas

### Services

- [x] **MPRIS**
- [x] **PulseAudio**
- [x] **Network**
- [x] **Bluetooth**
- [x] **Battery**
- [x] **Notification Daemon**
- [x] **Power Profiles**
- [x] **System Tray**
  - [x] GTK4 Adapter
- [x] Hyprland
- [x] **Cava**

### UI Components

- [x] **Component Library** - Base Relm4 widgets and containers
- [x] **Bar Modules**:
  - [x] Battery
  - [x] Media
  - [x] Volume
  - [x] Network
  - [x] Bluetooth
  - [x] Clock
  - [x] Microphone
  - [x] System tray
  - [x] Notification
  - [x] Dashboard
  - [x] Netstat
  - [x] RAM
  - [x] CPU
  - [x] CPU Temp
  - [x] Storage
  - [x] Separator
  - [x] Power
  - [x] World clock
  - [x] Weather
  - [x] Idle Inhibit
  - [x] Keyboard input
  - [x] Hyprland Window title
  - [x] Hyprland submap
  - [x] Hyprsunset
  - [x] Hyprland workspaces
  - [x] Custom Modules
  - [x] Cava

#### Scoped out

- **Updates**
  - Too much surface area and distro coupling
  - Will be achievable easily via custom modules

### Dropdown Interfaces

- [x] **Audio Panel**
- [x] **Network Panel**
- [x] **Bluetooth Panel**
- [x] **Battery Panel**
- [x] **Media Panel**
- [x] **Weather Panel**
- [x] **Calendar Panel**
- [x] **Dashboard**
- [x] **Notifications Panel**

### Additional Features

- [x] **Notifications**
- [x] **OSD**
- [ ] **Settings Dialog (WIP)**

## Configuration

Configuration lives in `~/.config/wayle/config.toml` with live reloading.

```toml
[styling]
theme-provider = "wayle"

[styling.palette]
bg = "#16161e"
fg = "#c0caf5"
primary = "#7aa2f7"

[bar]
scale = 1
location = "top"
rounding = "sm"

[[bar.layout]]
monitor = "*"
left = ["clock"]
center = ["media"]
right = ["battery"]

[modules.clock]
format = "%H:%M"
icon-show = true
label-show = true
```

Config files can be split and imported for better organization:

```toml
# config.toml
imports = ["colors.toml", "modules/bar.toml"]

[bar]
location = "top"
```

CLI commands can also be used to modify, get or reset any property:

```bash
wayle config get bar.scale
wayle config set bar.location bottom
wayle config reset bar.scale
```

Once the project is finished, documentation will be added for all configurable
properties, in addition to having a settings GUI. Until then you can run the
following command to generate a reference config `config.toml.example` in your
config directory:

```bash
wayle config default
```

Editor intellisense is available via JSON Schema. Install
[Tombi](https://marketplace.visualstudio.com/items?itemName=tombi-toml.tombi)
for VSCode or the `tombi` LSP for Neovim. The schema is generated automatically
on startup.

This will give you auto-complete, config validation and other nice QoL features
for your config.toml (and other toml files).

```bash
wayle config schema
```

## Building

Install Rust via [rustup](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repository recursively and build:

```bash
git clone --recursive https://github.com/wayle-rs/wayle
cd wayle
cargo install --path crates/wayle-shell
cargo install --path wayle
```

Once Wayle is installed, you can set up the icons (temporary measure) and start
it via:

```bash
wayle icons setup
wayle panel start
```

## Installation

<details>
    <summary> <h3> NixOS - Flake </h3> </summary>

For systems running NixOS, wayle is available to be used as home-manager module
via a flake.

First, add wayle to your inputs in your `flake.nix` file:

```nix
# flake.nix
{
    # ...
    inputs = {
        # ...
        wayle = {
          url = "git+https://github.com/Jas-SinghFSU/wayle";
          inputs.nixpkgs.follows = "nixpkgs-unstable"; # Optional (not tested on stable nixpkgs)
        };
        # ...
    };
    # ...
}
```

Note, you must use the path `git+https://github.com/...` since this project uses
git submodules and the `github://...` path style does not support these.

Then, still in your `flake.nix` file, add wayle as a module to your home manager
config. How your home-manager is configured might be slightly different but in
this example home-manager is configured as a nix-module. The important part is
adding `inputs.wayle.homeManagerModules.default` to your home-manager shared
modules list.

```nix
# flake.nix
{
    # ...

    outputs = { nixpkgs, ... } @ inputs: {
        # ...
        nixosConfigurations.default = nixpkgs.lib.nixosSystem {
            # ...
            modules = [
                # ...
                inputs.home-manager.nixosModules.home-manager
                {
                    home-manager = {
                        # ...

                        sharedModules = [
                            # ...

                            # Adding wayle to the sharedModules list inside of
                            # your home manager module is the important part.
                            inputs.wayle.homeManagerModules.default
                        ];
                        # ...
                    };
                }
                # ...
            ];
        };

        # ...
    };

}
```

Finally, you can enable wayle in your `home.nix` file using:

```nix
# home.nix
{
    config,
    lib,
    pkgs,
    inputs, # Only needed to install the wayle package manually.
    ...
}: {
    # ...

    services.wayle = {
        enable = true;
        settings = {
            # Put your wayle configuration here.
        };

        # Uncomment this if you don't want wayle to start automatically:
        # systemd.enable = false;
    };

    # -- Optional --
    # If you would like to access the `wayle` and `wayle-shell` commands from
    # your command line, then remember to add wayle as a package to your home
    # environment using the following.
    #
    # This is required if you set `services.wayle.systemd.enable = false`
    # above. Otherwise you will have no way to start wayle.
    home.packages = with pkgs; [
        # ...
        inputs.wayle.packages.${stdenv.hostPlatform.system}.default
        # ...
    ];

    # ...
}
```

#### Notes on using NixOS

##### Icons

This nix package will automatically install the icons for you so you don't need
to worry about running `wayle icons setup`.

##### Live Config Reloading

NixOS will generate the wayle `config.toml` file for you using the config you
wrote for wayle in your `home.nix` file. Unfortunately, this misses out on
wayle's live config reloading feature. To work around this and configure wayle
without rebuilding your home-manager config everytime and to allow utilizing the
`wayle config` the following steps are recommended:

1. Move the nix-made symlink config file to a new path and make a clone of it to
   be used as a starting point with:
   ```bash
   mv ~/.config/wayle/config.toml{,.bak}
   cp ~/.config/wayle/config.toml{.bak,}
   ```

2. Modify the config file at `~/.config/wayle/config.toml` either manually or by
   using the `wayle config <cmd>` commands until you are happy with your
   configuration.

3. Translate your new config from the TOML syntax of the
   `~/.config/wayle/config.toml` file into nix syntax in your `home.nix` file at
   `services.wayle.settings = { # settings go here }`.

4. Move the original config symlink back (this will overwrite whatever changes
   you may have made and return you to the config defined by your nix config).
   ```bash
   mv ~/.config/wayle/config.toml{.bak,}
   ```

5. Rebuild your home-manager config and the new settings will be applied.

This method allows you to configure wayle efficiently and benefit from its live
config reloading and command line features, while still declaratively writing
your config using nix.

**CAUTION:** Home manager will automatically move any existing config to a
`config.toml.hm-bak`, or similar, backup file name. This is expected, however,
wayle will automatically create a _new_ `config.toml` file if you moved the
symlink make by home-manager and didn't move it back. If this occurs,
home-manager will try to move the new `config.toml` (non-symlink) file to
`config.toml.hm-bak` (or similar) _again_ and fail because that back up file
already exists. If you are rebuilding with nix, and it is failing, check if your
`~/.config/wayle` directory contains these non-symlinked files and remove or
re-backup them to a different file name. Home assistant seems to give very
little to no feedback when this error occurs.

</details>

## Icons

Wayle uses GTK symbolic icons that support CSS color theming.

To manually manage icons:

```bash
# Install bundled icons (automatic on first launch)
wayle icons setup

# Install additional icons from CDN sources
wayle icons install tabler home settings bell
wayle icons install simple-icons firefox spotify

# See all available sources
wayle icons install --help
```

Icons are installed to `~/.local/share/wayle/icons/` as GTK symbolic icons.

## License

MIT
