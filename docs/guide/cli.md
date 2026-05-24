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

Shell completions for bash, fish, and zsh:

```sh
wayle completions fish > ~/.config/fish/completions/wayle.fish
```
