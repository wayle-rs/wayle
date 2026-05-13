---
title: Getting started on NixOS
---

# Getting started on NixOS

Requires NixOS unstable or 25.11. Note that Wayle was added only recently, so update to the latest version before trying to install it.

## Install package

Wayle is available as `pkgs.wayle` package, but if you use home-manager there is a module so you don't have to install the package manually.

<details>

<summary> Details regarding dependency behaviour
Nix tries to be as unopinionated as possible, so optional dependencies are not installed, and users have to handle it themselves.

### Package
For instance, the wayle nixpkg doesn't include the aww daemon, so it'll need to be enabled seperately if you wish to use wayle to
manage wallpapers.

### Module
Dependencies required by configuration set within services.wayle.settings will be installed, controlled by services.wayle.autoInstallDependencies
option.
</details>

> [!CAUTION]
> Using the package and configuring wayle via the gui on flake+home-manager install can lead to DBUS/dbus-broker issues, especially if trying to
> configure a non existent dependency.  If apps fail to launch after several hours of uptime and you see alerts such as 
> 'dbus-broker[1217]: Peer :1.369 is being disconnected as it does not have the resources to receive a reply it requested.' 
> then you should switch to the following declarative build.

## home-manager usage

```nix
# put this directly into your home-manager config or into a home-manager import
{ pkgs, lib, ... }:
{
  services.wayle = {
    enable = true;

    # tip: you can automatically translate your TOML config to Nix by running
    # nix-instantiate --eval --expr 'builtins.fromTOML (builtins.readFile ./config.toml)' | nixfmt
    settings = {
      bar = {
        layout = [ 
          { # add more attribute sets with different monitors if wayle should have different layouts on each
            monitor = "DP-1"; # replace "DP-1" with "*" for this layout across all monitors
            show = true;
            center = [
              "clock"
              "weather"
            ];
            left = [
              "dashboard"
            ];
            right = [
              "volume"
            ];
          }  # <-- this is a 'list' of 'attribute sets',no semi-colons after the closing braces inside the list <--
        ];
      };
      modules = {
        clock = {
         format = "%H:%M:%S";
         dropdown-show-seconds = false;
         };
        weather = {
          location = "Denver";
          units = "imperial";
        };
      };
      osd = {
        monitor = "DP-1";
      };
      styling = {
        palette = {
          bg = "#282a36";
          blue = "#8be9fd";
          elevated = "#44475a";
          fg = "#f8f8f2";
          fg-muted = "#6272a4";
          green = "#50fa7b";
          primary = "#bd93f9";
          red = "#ff5555";
          surface = "#343746";
          yellow = "#f1fa8c";
        };
      }; #the following wallpaper option can be ommited if you're not using wayle's wallpaper engine
      wallpaper = {
        cycling-directory = "/home/horsey/Pictures/Backgrounds/1/";
        cycling-mode = "shuffle";
        engine-enabled = true;
      };
    };
  };
}
```

## Settings GUI

```sh
wayle panel settings
```

This launches `wayle-settings`, which edits the same config the shell reads. Changes apply live. Anything the GUI doesn't cover can still be edited by hand in `config.toml`.

After configuring Wayle using GUI, there should be a new `.config/wayle/runtime.toml` file. To automatically convert it to Nix, run
```sh
cd ~/.config/wayle
nix-instantiate --eval --expr '(builtins.fromTOML (builtins.readFile ./config.toml)) // (builtins.fromTOML (builtins.readFile ./runtime.toml))' | nixfmt
```
If one of those files does not exist run the following command, replacing `config.toml` with `runtime.toml` if appropriate.
```sh
nix-instantiate --eval --expr 'builtins.fromTOML (builtins.readFile ./config.toml)' | nixfmt
```

Then you can copy-paste this into the `services.wayle.settings` option provided by the home-manager module.

## Configuration

If you want to edit the raw `config.toml`, refer to the [Editing config](/guide/editing-config) page (note that Tombi doesn't work when config is symlinked to `/nix/store`, you need to create a normal file first).
