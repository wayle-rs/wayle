---
title: Editing config
---

# Editing config

Wayle reads configuration from `~/.config/wayle/config.toml`. All fields have defaults, so `config.toml` may be empty; declare only the fields that should override a default.

The `wayle config` CLI and the settings GUI write to `~/.config/wayle/runtime.toml` rather than `config.toml`. For each field, Wayle uses the first value defined among these sources:

1. `runtime.toml` - overrides written by `wayle config` or the settings GUI.
2. `config.toml` - values declared by hand.
3. The built-in default.

A minimal override:

```toml
[bar]
scale = 1.25

[modules.clock]
format = "%H:%M"
```

Every supported key is documented in the [config reference](/config/).

## Imports

`config.toml` may declare a top-level `imports` array to load additional TOML files. Files referenced through `imports` may themselves declare `imports`, forming a chain:

```toml
imports = ["themes/nord.toml", "modules/clock.toml"]

[bar]
scale = 1.25
```

Paths are resolved relative to the importing file's directory; the `.toml` extension may be omitted. Imports are merged in declaration order, then the importing file is overlaid on top. Tables merge key by key; scalars and arrays in the overlay replace the corresponding value in the base. The merged result becomes the `config.toml` layer described above.

`runtime.toml` does not resolve imports. Circular chains are rejected at load; the previous valid configuration remains active and the error is recorded in the log.

## Editor setup

On startup, Wayle writes a JSON Schema for the configuration to `~/.config/wayle/schema.json`. Any TOML language server with JSON Schema support can use this file for completion, hover documentation, and validation. The schema is generated from the installed binary and matches the version of Wayle on disk.

[Tombi](https://tombi-toml.github.io/tombi/) is one such server. The Tombi extension is available in the VS Code marketplace; the `tombi` LSP binary runs under Neovim, Helix, and Zed. Configure the server to associate `~/.config/wayle/schema.json` with `config.toml`.

## Live reload

Wayle watches the configuration directory. Changes to `config.toml` trigger an in-process reload; a shell restart is not required. Invalid configuration is rejected, the previous valid state is retained, and parse or validation errors are recorded in the log.

## Editing from the CLI

The `wayle config` subcommand reads and writes individual fields by dotted path:

```bash
wayle config get bar.scale
wayle config set modules.clock.format "%H:%M"
wayle config reset modules.clock.format
```

`set` writes to `~/.config/wayle/runtime.toml`; `config.toml` is never modified by the CLI or GUI Settings dialog. `reset` removes the runtime override for the given path, reverting the field to the value declared in `config.toml` or to the built-in default.

## Printing the default configuration

`wayle config default --stdout` prints every key with its default value to standard output. Without `--stdout`, the command writes `config.toml.example` to the configuration directory; `config.toml` is not modified. `wayle config schema --stdout` prints the JSON Schema in the same manner.
