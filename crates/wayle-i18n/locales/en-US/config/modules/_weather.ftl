### Wayle Configuration - Weather Module

## Weather Module Configuration

settings-modules-weather-provider = Provider
    .description = Weather data provider

settings-modules-weather-location = Location
    .description = City name or "lat,lon" coordinates

settings-modules-weather-units = Temperature Unit
    .description = Metric (Celsius) or Imperial (Fahrenheit)

settings-modules-weather-format = Label Format
    .description = Format string with placeholders: {"{{ temp }}"}, {"{{ temp_unit }}"}, {"{{ feels_like }}"}, {"{{ condition }}"}, {"{{ humidity }}"}, {"{{ wind_speed }}"}, {"{{ wind_dir }}"}, {"{{ high }}"}, {"{{ low }}"}

settings-modules-weather-refresh-interval = Refresh Interval
    .description = Polling interval in seconds

settings-modules-weather-visual-crossing-key = Visual Crossing Key
    .description = API key for Visual Crossing. Use $VAR_NAME to reference .env variables

settings-modules-weather-weatherapi-key = WeatherAPI Key
    .description = API key for WeatherAPI.com. Use $VAR_NAME to reference .env variables

settings-modules-weather-icon-name = Fallback Icon
    .description = Icon shown when weather data unavailable

settings-modules-weather-border-show = Show Border
    .description = Display border around button

settings-modules-weather-border-color = Border Color
    .description = Border color token

settings-modules-weather-icon-show = Show Icon
    .description = Display module icon

settings-modules-weather-icon-color = Icon Color
    .description = Icon foreground color

settings-modules-weather-icon-bg-color = Icon Background
    .description = Icon container background color

settings-modules-weather-label-show = Show Label
    .description = Display temperature label

settings-modules-weather-label-color = Label Color
    .description = Label text color

settings-modules-weather-label-max-length = Label Max Length
    .description = Max characters before truncation

settings-modules-weather-button-bg-color = Button Background
    .description = Button background color

settings-modules-weather-right-click = Right Click
    .description = Shell command on right click

settings-modules-weather-middle-click = Middle Click
    .description = Shell command on middle click

settings-modules-weather-scroll-up = Scroll Up
    .description = Shell command on scroll up

settings-modules-weather-scroll-down = Scroll Down
    .description = Shell command on scroll down


settings-modules-weather-time-format = Time Format
    .description = Time display format for forecasts

settings-modules-weather-left-click = Left Click
    .description = Action on left click
