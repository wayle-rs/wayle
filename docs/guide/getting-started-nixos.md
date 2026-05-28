---
title: Getting started on NixOS
---

# Getting started on NixOS

Requires NixOS unstable or 25.11. Note that Wayle was added only recently, so update to the latest version before trying to install it.

## Install package

Wayle is available as `pkgs.wayle` package. But if you use the home-manager module, you don't have to install the package manually.

## home-manager usage

```nix
# put this into your home-manager config
{ pkgs, lib, ... }:
{
  # then you can use it as a normal program
  services.wayle = {
    enable = true;

    # tip: you can automatically translate your TOML config to Nix by running
    # nix-instantiate --eval --expr 'builtins.fromTOML (builtins.readFile ./config.toml)' | nixfmt
    settings = {
      modules = {
        clock = {
          format = "%H:%M:%S";
          dropdown-show-seconds = false;
        };
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
Or if one of the files does not exist (you can replace `config.toml` with `runtime.toml`):
```sh
nix-instantiate --eval --expr 'builtins.fromTOML (builtins.readFile ./config.toml)' | nixfmt
```

Then you can copy-paste this into your `services.wayle.settings` home-manager option.

## Configuration

If you want to edit the raw `config.toml`, refer to the [Editing config](/guide/editing-config) page (note that Tombi doesn't work when config is symlinked to `/nix/store`, you need to create a normal file first).
