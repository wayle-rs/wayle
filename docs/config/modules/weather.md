---
title: weather
outline: [2, 3]
---

# weather

<div v-pre>

Current conditions with hourly and daily forecasts in a dropdown.

Add it to your layout with `weather`:

```toml
[[bar.layout]]
monitor = "*"
right = ["weather"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `provider` | [`WeatherProvider`](/config/types#weather-provider) | `"open-meteo"` | Weather data provider. |
| `location` | string | `"San Francisco"` | Location for weather data (city name or "lat,lon" coordinates). |
| `units` | [`TemperatureUnit`](/config/types#temperature-unit) | `"metric"` | Temperature unit. |
| `format` | string | `"{{ temp }}{{ temp_unit }}"` | Format string for the label. |
| `time-format` | [`TimeFormat`](/config/types#time-format) | `"12h"` | Time display format for sunrise/sunset and hourly forecast. |
| `refresh-interval-seconds` | u32 | `1800` | Polling interval in seconds. |
| `visual-crossing-key` | string or null | `null` | Visual Crossing API key. Supports `$VAR_NAME` syntax to reference environment variables from `.*.env` files in the config directory. |
| `weatherapi-key` | string or null | `null` | WeatherAPI.com API key. Supports `$VAR_NAME` syntax to reference environment variables from `.*.env` files in the config directory. |
| `icon-name` | string | `"ld-sun-symbolic"` | Fallback icon for weather. |
| `border-show` | bool | `false` | Display border around button. |
| `icon-show` | bool | `true` | Display module icon. |
| `label-show` | bool | `true` | Display temperature label. |
| `label-max-length` | u32 | `0` | Max label characters before truncation with ellipsis. Set to 0 to disable. |

::: details More about `format`

#### Placeholders

- `{{ temp }}` - Current temperature (e.g., "72")
- `{{ temp_unit }}` - Temperature unit symbol ("°F" or "°C")
- `{{ feels_like }}` - Feels-like temperature
- `{{ condition }}` - Weather condition text (e.g., "Cloudy")
- `{{ humidity }}` - Humidity percentage (e.g., "65%")
- `{{ wind_speed }}` - Wind speed with unit (e.g., "12 km/h")
- `{{ wind_dir }}` - Wind direction (e.g., "NW")
- `{{ high }}` - Today's high temperature
- `{{ low }}` - Today's low temperature

#### Examples

- `"{{ temp }}{{ temp_unit }}"` - "22°C"
- `"{{ temp }}{{ temp_unit }} {{ condition }}"` - "22°C Partly Cloudy"
- `"{{ temp }}{{ temp_unit }} H:{{ high }} L:{{ low }}"` - "22°C H:25 L:18"

:::

## Colors

| Field | Type | Default | Description |
|---|---|---|---|
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-accent"` | Border color token. |
| `icon-color` | [`ColorValue`](/config/types#color-value) | `"auto"` | Icon foreground color. Auto selects based on variant for contrast. |
| `icon-bg-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Icon container background color token. |
| `label-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Label text color token. |
| `button-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Button background color token. |

## Click actions

| Field | Type | Default | Description |
|---|---|---|---|
| `left-click` | [`ClickAction`](/config/types#click-action) | `"dropdown:weather"` | Action on left click. |
| `right-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on right click. |
| `middle-click` | [`ClickAction`](/config/types#click-action) | `""` | Action on middle click. |
| `scroll-up` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll up. |
| `scroll-down` | [`ClickAction`](/config/types#click-action) | `""` | Action on scroll down. |

## Default configuration

```toml
[modules.weather]
provider = "open-meteo"
location = "San Francisco"
units = "metric"
format = "{{ temp }}{{ temp_unit }}"
time-format = "12h"
refresh-interval-seconds = 1800
icon-name = "ld-sun-symbolic"
border-show = false
border-color = "border-accent"
icon-show = true
icon-color = "auto"
icon-bg-color = "accent"
label-show = true
label-color = "accent"
label-max-length = 0
button-bg-color = "bg-surface-elevated"
left-click = "dropdown:weather"
right-click = ""
middle-click = ""
scroll-up = ""
scroll-down = ""
```


</div>
