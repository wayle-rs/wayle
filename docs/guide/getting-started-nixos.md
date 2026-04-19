---
title: Getting started on NixOS
---

# Getting started on NixOS

Requires NixOS unstable or 25.11 (the stable version requires more tweaks at the moment). Note that Wayle was added only recently, so update to the latest version before trying to install it.

## Install package

Wayle is available as `pkgs.wayle` package (only if you are on unstable). But if you use the home-manager module, you don't have to install the package manually.

## home-manager module

Create a new folder named `wayle` somewhere in your config and put there these files:
- `package.nix` - only if you are on NixOS 25.11; copy file from 25.11 PR ([link](https://raw.githubusercontent.com/PerchunPak/nixpkgs/refs/heads/wayle-stable/pkgs/by-name/wa/wayle/package.nix))
- `module.nix` - only if you are on NixOS 25.11; copy home-manager module ([link](https://raw.githubusercontent.com/nix-community/home-manager/refs/heads/master/modules/services/wayle.nix))
- `default.nix`, this is where you can configure Wayle, put this there:

```nix
# put this into your home-manager config
{ pkgs, lib, ... }:
{
  imports = [
    ./module.nix
    # if you are on 25.11, also add this
    # (lib.mkAliasOptionModule [ "services" "swww" ] [ "services" "awww" ])
  ];

  # then you can use it as a normal program
  services.wayle = {
    enable = true;
    # if you are on 25.11, also add this
    # package = pkgs.callPackage ./package.nix { };

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

Then you can import the folder as a usual module (`imports = [ ./wayle ];` where `./wayle` is the path to the folder you created before)

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
