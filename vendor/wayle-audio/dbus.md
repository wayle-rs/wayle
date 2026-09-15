# Audio D-Bus Interface

Control PulseAudio volume, mute, and device selection.

- **Service:** `com.wayle.Audio1`
- **Path:** `/com/wayle/Audio`

## Methods

### Volume Control

| Method               | Arguments  | Returns | Description                    |
| -------------------- | ---------- | ------- | ------------------------------ |
| `SetOutputVolume`    | `d volume` | `d`     | Set output volume (0-100)      |
| `AdjustOutputVolume` | `d delta`  | `d`     | Adjust volume by delta (+/- %) |
| `SetInputVolume`     | `d volume` | `d`     | Set input/mic volume (0-100)   |
| `AdjustInputVolume`  | `d delta`  | `d`     | Adjust input volume by delta   |

### Mute Control

| Method             | Arguments | Returns | Description                           |
| ------------------ | --------- | ------- | ------------------------------------- |
| `SetOutputMute`    | `b muted` | -       | Set output mute state                 |
| `ToggleOutputMute` | -         | `b`     | Toggle output mute, returns new state |
| `SetInputMute`     | `b muted` | -       | Set input/mic mute state              |
| `ToggleInputMute`  | -         | `b`     | Toggle input mute, returns new state  |

### Device Selection

| Method                 | Arguments | Returns  | Description                              |
| ---------------------- | --------- | -------- | ---------------------------------------- |
| `SetDefaultSink`       | `u index` | -        | Set default output device by index       |
| `SetDefaultSource`     | `u index` | -        | Set default input device by index        |
| `ListSinks`            | -         | `a(uss)` | List outputs: (index, name, description) |
| `ListSources`          | -         | `a(uss)` | List inputs: (index, name, description)  |
| `GetDefaultSinkInfo`   | -         | `a{ss}`  | Get default output device info           |
| `GetDefaultSourceInfo` | -         | `a{ss}`  | Get default input device info            |

## Properties

| Property        | Type | Access | Description                   |
| --------------- | ---- | ------ | ----------------------------- |
| `OutputVolume`  | `d`  | read   | Current output volume (0-100) |
| `OutputMuted`   | `b`  | read   | Output mute state             |
| `InputVolume`   | `d`  | read   | Current input volume (0-100)  |
| `InputMuted`    | `b`  | read   | Input mute state              |
| `DefaultSink`   | `s`  | read   | Name of default output device |
| `DefaultSource` | `s`  | read   | Name of default input device  |
| `SinkCount`     | `u`  | read   | Number of output devices      |
| `SourceCount`   | `u`  | read   | Number of input devices       |
