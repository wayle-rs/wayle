### Bar Module Labels

## Network
bar-network-connecting = Connecting...
bar-network-disconnected = Disconnected
bar-network-wired = Wired
bar-network-wifi-fallback = WiFi
bar-network-no-wifi = No WiFi
bar-network-no-ethernet = No Ethernet
bar-network-offline = Offline

## Battery
bar-battery-unavailable = N/A

## Bluetooth
bar-bluetooth-disabled = Off
bar-bluetooth-disconnected = Disconnected
bar-bluetooth-connected-count = { $count ->
    [one] { $count } Connected
   *[other] { $count } Connected
}

## Window Title
bar-window-title-empty = Desktop

## Idle Inhibit
bar-idle-inhibit-on = On
bar-idle-inhibit-off = Off

## Keybind Mode
bar-keybind-mode-default = default

## Hyprsunset
bar-hyprsunset-on = On
bar-hyprsunset-off = Off

## Storage
bar-storage-multiple = Multiple

## Media
bar-media-playing = Playing
bar-media-paused = Paused
bar-media-stopped = Stopped

### Notification Dropdown

notification-dropdown-title = Notifications
notification-dropdown-empty-title = All Caught Up
notification-dropdown-empty-description = No new notifications
notification-dropdown-clear-all = Clear All
notification-dropdown-dnd-label = Do Not Disturb
notification-dropdown-group-clear = Clear
notification-dropdown-group-more = { $count } more
notification-dropdown-unknown-app = Unknown
notification-dropdown-time-just-now = Just now
notification-dropdown-time-minutes-ago = { $minutes }m ago
notification-dropdown-time-hours-ago = { $hours }h ago

### Notification Popup

notification-popup-unknown-app = Unknown
notification-popup-time-just-now = Just now
notification-popup-time-minutes-ago = { $minutes }m ago
notification-popup-time-hours-ago = { $hours }h ago

### On-Screen Display

## Toggle labels
osd-caps-lock = Caps Lock
osd-num-lock = Num Lock
osd-scroll-lock = Scroll Lock

## Toggle state
# { $label } is the toggle name (e.g., "Caps Lock")
osd-toggle-on = { $label } On
osd-toggle-off = { $label } Off

### Weather Conditions

weather-clear = Clear
weather-partly-cloudy = Partly Cloudy
weather-cloudy = Cloudy
weather-overcast = Overcast
weather-mist = Mist
weather-fog = Fog
weather-light-rain = Light Rain
weather-rain = Rain
weather-heavy-rain = Heavy Rain
weather-drizzle = Drizzle
weather-light-snow = Light Snow
weather-snow = Snow
weather-heavy-snow = Heavy Snow
weather-sleet = Sleet
weather-thunderstorm = Thunderstorm
weather-windy = Windy
weather-hail = Hail
weather-unknown = Unknown

### Audio Dropdown

dropdown-audio-title = Audio
dropdown-audio-output = Output
dropdown-audio-input = Input
dropdown-audio-output-devices = Output Devices
dropdown-audio-input-devices = Input Devices
dropdown-audio-app-volume = Application Volume
dropdown-audio-no-device = No devices found
dropdown-audio-no-devices-title = No Audio Devices
dropdown-audio-no-devices-description = No audio output or input devices found
dropdown-audio-no-apps = No applications playing audio
dropdown-audio-settings = Audio Settings

### Battery Dropdown

dropdown-battery-title = Battery

## Hero States
dropdown-battery-on-battery = On Battery
dropdown-battery-charging = Charging
dropdown-battery-plugged-in = Plugged In
dropdown-battery-critical = Critical

## Time Display
dropdown-battery-duration-hm = { $hours }h { $minutes }m
dropdown-battery-duration-m = { $minutes }m
dropdown-battery-time-remaining = { $duration } remaining
dropdown-battery-time-until-full = { $duration } until full

## Details
dropdown-battery-draw = Draw
dropdown-battery-input = Input
dropdown-battery-input-watts = { $watts } input
dropdown-battery-capacity = Capacity
dropdown-battery-charged = Charged
dropdown-battery-health = Health

## Charge Limit
dropdown-battery-charge-limit = Charge Limit
dropdown-battery-limit-to = Limit to { $threshold }%
dropdown-battery-resumes-at = Resumes charging at { $threshold }%
dropdown-battery-charge-limit-not-supported = Charge limit not supported on this device

## Power Profile
dropdown-battery-power-profile = Power Profile
dropdown-battery-profile-saver = Saver
dropdown-battery-profile-balanced = Balanced
dropdown-battery-profile-performance = Performance
dropdown-battery-power-profile-not-available = Power profiles daemon must be active

## No Battery
dropdown-battery-no-battery-title = No Battery Detected
dropdown-battery-no-battery-description = Running on AC power

### Bluetooth Dropdown

dropdown-bluetooth-title = Bluetooth
dropdown-bluetooth-my-devices = My Devices
dropdown-bluetooth-available-devices = Available Devices
dropdown-bluetooth-connected = Connected
dropdown-bluetooth-scanning = Scanning
dropdown-bluetooth-connect = Connect
dropdown-bluetooth-disconnect = Disconnect
dropdown-bluetooth-forget = Forget
dropdown-bluetooth-pair = Pair
dropdown-bluetooth-cancel = Cancel
dropdown-bluetooth-confirm = Confirm
dropdown-bluetooth-reject = Reject
dropdown-bluetooth-try-again = Try Again
dropdown-bluetooth-allow = Allow
dropdown-bluetooth-deny = Deny

## Device Status
dropdown-bluetooth-battery = { $percent }%
dropdown-bluetooth-paired = Paired
dropdown-bluetooth-not-connected = Not connected
dropdown-bluetooth-new-device = New device
dropdown-bluetooth-status-connecting = Connecting...
dropdown-bluetooth-status-disconnecting = Disconnecting...
dropdown-bluetooth-status-forgetting = Removing...

## Device Types - Computer
dropdown-bluetooth-type-computer = Computer
dropdown-bluetooth-type-desktop = Desktop
dropdown-bluetooth-type-server = Server
dropdown-bluetooth-type-laptop = Laptop
dropdown-bluetooth-type-handheld = Handheld PC
dropdown-bluetooth-type-palm = Palm PC
dropdown-bluetooth-type-wearable-computer = Wearable computer
dropdown-bluetooth-type-computer-tablet = Tablet

## Device Types - Phone
dropdown-bluetooth-type-phone = Phone
dropdown-bluetooth-type-cellular = Cellular
dropdown-bluetooth-type-cordless = Cordless
dropdown-bluetooth-type-smartphone = Smart phone
dropdown-bluetooth-type-modem = Modem

## Device Types - Network
dropdown-bluetooth-type-network = Access point

## Device Types - Audio/Video
dropdown-bluetooth-type-headset = Headset
dropdown-bluetooth-type-handsfree = Hands-free
dropdown-bluetooth-type-microphone = Microphone
dropdown-bluetooth-type-loudspeaker = Loudspeaker
dropdown-bluetooth-type-headphones = Headphones
dropdown-bluetooth-type-portable-audio = Portable audio
dropdown-bluetooth-type-car-audio = Car audio
dropdown-bluetooth-type-set-top-box = Set-top box
dropdown-bluetooth-type-hifi = Hi-Fi audio
dropdown-bluetooth-type-vcr = VCR
dropdown-bluetooth-type-video-camera = Video camera
dropdown-bluetooth-type-camcorder = Camcorder
dropdown-bluetooth-type-video-monitor = Video monitor
dropdown-bluetooth-type-video-display = Video display and loudspeaker
dropdown-bluetooth-type-video-conferencing = Video conferencing
dropdown-bluetooth-type-gaming = Gaming/Toy
dropdown-bluetooth-type-audio-video = Audio/Video

## Device Types - Peripheral
dropdown-bluetooth-type-keyboard = Keyboard
dropdown-bluetooth-type-mouse = Pointing device
dropdown-bluetooth-type-combo-keyboard = Keyboard/Pointing device
dropdown-bluetooth-type-joystick = Joystick
dropdown-bluetooth-type-gamepad = Gamepad
dropdown-bluetooth-type-remote = Remote control
dropdown-bluetooth-type-sensing = Sensing device
dropdown-bluetooth-type-tablet = Digitizer tablet
dropdown-bluetooth-type-card-reader = Card reader
dropdown-bluetooth-type-peripheral = Peripheral

## Device Types - Imaging
dropdown-bluetooth-type-imaging = Imaging
dropdown-bluetooth-type-display = Display
dropdown-bluetooth-type-camera = Camera
dropdown-bluetooth-type-scanner = Scanner
dropdown-bluetooth-type-printer = Printer

## Device Types - Wearable
dropdown-bluetooth-type-wearable = Wearable
dropdown-bluetooth-type-wrist-watch = Wrist watch
dropdown-bluetooth-type-pager = Pager
dropdown-bluetooth-type-jacket = Jacket
dropdown-bluetooth-type-helmet = Helmet
dropdown-bluetooth-type-glasses = Glasses

## Device Types - Toy
dropdown-bluetooth-type-toy = Toy
dropdown-bluetooth-type-robot = Robot
dropdown-bluetooth-type-vehicle = Vehicle
dropdown-bluetooth-type-doll = Doll
dropdown-bluetooth-type-controller = Controller
dropdown-bluetooth-type-game = Game

## Device Types - Other
dropdown-bluetooth-type-health = Health
dropdown-bluetooth-type-unknown = Bluetooth device

## Service Names (for RequestServiceAuthorization)
dropdown-bluetooth-service-serial-port = Serial Port
dropdown-bluetooth-service-lan-access = LAN Access
dropdown-bluetooth-service-dialup-networking = Dialup Networking
dropdown-bluetooth-service-object-push = Object Push
dropdown-bluetooth-service-file-transfer = File Transfer
dropdown-bluetooth-service-headset = Headset Audio
dropdown-bluetooth-service-audio-source = Audio Source
dropdown-bluetooth-service-audio-sink = Audio Sink
dropdown-bluetooth-service-remote-control = Remote Control
dropdown-bluetooth-service-audio-distribution = Audio Streaming
dropdown-bluetooth-service-handsfree = Hands-Free
dropdown-bluetooth-service-network-access = Network Access
dropdown-bluetooth-service-input-device = Input Device
dropdown-bluetooth-service-sim-access = SIM Access
dropdown-bluetooth-service-phonebook = Phonebook Access
dropdown-bluetooth-service-messaging = Messaging
dropdown-bluetooth-service-unknown = Bluetooth Service
dropdown-bluetooth-service-proprietary = Bluetooth Service

## Empty States
dropdown-bluetooth-no-devices-title = No Devices Found
dropdown-bluetooth-no-devices-description = Make sure your device is in pairing mode
dropdown-bluetooth-off-title = Bluetooth is Off
dropdown-bluetooth-off-description = Turn on Bluetooth to connect devices
dropdown-bluetooth-no-adapter-title = No Bluetooth Adapter
dropdown-bluetooth-no-adapter-description = No Bluetooth adapter was detected
dropdown-bluetooth-no-nearby = No other devices nearby
dropdown-bluetooth-no-new = No new devices found

## Pairing
# DisplayPinCode: PIN label above the code
dropdown-bluetooth-pairing-enter-pin = Enter this PIN on the device
# DisplayPinCode: instruction below the code
dropdown-bluetooth-pairing-type-on-device = Type the PIN on the device, then press Enter
# RequestPasskey: user enters the PIN shown on the other device
dropdown-bluetooth-pairing-enter-shown-pin = Enter the PIN displayed on the device
# RequestConfirmation: both sides show same code, user confirms match
dropdown-bluetooth-pairing-confirm-code = Confirm that this code matches the one on the device:
# DisplayPasskey: progress while other device types passkey
dropdown-bluetooth-pairing-entering = { $entered } of { $total } digits entered
# RequestAuthorization: "Just Works" pairing, user accepts or denies
dropdown-bluetooth-pairing-allow-pairing = Allow this device to pair?
# AuthorizeService: post-pairing service authorization
dropdown-bluetooth-pairing-service-allow = Allow this device to access the requested service?
# RequestPinCode: legacy PIN entry
dropdown-bluetooth-pairing-enter-legacy-pin = Enter the PIN for this device
# Placeholder text for the legacy PIN input field
dropdown-bluetooth-pairing-pin-placeholder = PIN
dropdown-bluetooth-pairing-common-pins = Common PINs: 0000, 1234, 1111
# Pairing failure
dropdown-bluetooth-pairing-failed = Pairing failed. The PIN may be incorrect or the request timed out.

### Brightness Dropdown

dropdown-brightness-title = Brightness

## Empty State
dropdown-brightness-empty-title = No backlight detected
dropdown-brightness-empty-description = Brightness control is available only for internal displays.

### Calendar Dropdown

dropdown-calendar-title = Calendar

## Clock Display

cal-day-sunday = Sunday
cal-day-monday = Monday
cal-day-tuesday = Tuesday
cal-day-wednesday = Wednesday
cal-day-thursday = Thursday
cal-day-friday = Friday
cal-day-saturday = Saturday

# { $month } - localized month name, { $day } - day number, { $year } - four-digit year
cal-clock-date-rest = , { $month } { $day }, { $year }

## Calendar Widget

cal-today = Today

# { $month } - localized month name, { $year } - four-digit year
cal-month-year = { $month } { $year }

cal-weekday-sun = Su
cal-weekday-mon = Mo
cal-weekday-tue = Tu
cal-weekday-wed = We
cal-weekday-thu = Th
cal-weekday-fri = Fr
cal-weekday-sat = Sa

cal-month-january = January
cal-month-february = February
cal-month-march = March
cal-month-april = April
cal-month-may = May
cal-month-june = June
cal-month-july = July
cal-month-august = August
cal-month-september = September
cal-month-october = October
cal-month-november = November
cal-month-december = December

### Dashboard Dropdown

dropdown-dashboard-title = Dashboard
dropdown-dashboard-open-settings = Open Settings

## Quick Actions
dropdown-dashboard-wifi = WiFi
dropdown-dashboard-bluetooth = Bluetooth
dropdown-dashboard-airplane = Airplane Mode
dropdown-dashboard-dnd = Do Not Disturb
dropdown-dashboard-idle-inhibit = Idle Inhibit
dropdown-dashboard-power-saver = Power Saver

## Controls
dropdown-dashboard-volume = Volume
dropdown-dashboard-no-device = No output device

## Media
dropdown-dashboard-now-playing = Now Playing
dropdown-dashboard-no-media-title = No Media
dropdown-dashboard-no-media-description = No media playing

## Battery
dropdown-dashboard-battery = Battery
dropdown-dashboard-battery-charging = Charging
dropdown-dashboard-battery-discharging = Discharging
dropdown-dashboard-battery-fully-charged = Fully Charged
dropdown-dashboard-battery-pending-charge = Pending Charge
dropdown-dashboard-battery-pending-discharge = Pending Discharge
dropdown-dashboard-battery-empty = Empty
dropdown-dashboard-battery-unknown = Unknown
dropdown-dashboard-battery-time-hm = ~{ $hours }h { $minutes }m
dropdown-dashboard-battery-time-m = ~{ $minutes }m
dropdown-dashboard-battery-profile-saver = Power Saver
dropdown-dashboard-battery-profile-balanced = Balanced
dropdown-dashboard-battery-profile-performance = Performance

## Network
dropdown-dashboard-network = Network
dropdown-dashboard-network-disconnected = Disconnected
dropdown-dashboard-network-connecting = Connecting...
dropdown-dashboard-network-ethernet = Ethernet
dropdown-dashboard-network-wifi-off = WiFi Off
dropdown-dashboard-network-speed-kbs = KB/s
dropdown-dashboard-network-speed-mbs = MB/s

## System Stats
dropdown-dashboard-system = System
dropdown-dashboard-cpu = CPU
dropdown-dashboard-ram = RAM
dropdown-dashboard-disk = Disk
dropdown-dashboard-temp = CPU Temp

## User Session
dropdown-dashboard-lock = Lock
dropdown-dashboard-logout = Log Out
dropdown-dashboard-reboot = Reboot
dropdown-dashboard-power-off = Power Off

### Media Dropdown

dropdown-media-title = Now Playing
dropdown-media-no-player-title = No Media Playing
dropdown-media-no-player-description = Start playing media in any app to control it here
dropdown-media-sources = Media Sources
dropdown-media-unknown-title = Unknown Title
dropdown-media-unknown-artist = Unknown Artist
dropdown-media-unknown-album = Unknown Album

### Network Dropdown

dropdown-network-title = Network
dropdown-network-active-connections = Active Connections
dropdown-network-active-connection = Active Connection
dropdown-network-available = Available Networks
dropdown-network-connected = Connected
dropdown-network-connecting = Connecting
dropdown-network-connect = Connect
dropdown-network-disconnect = Disconnect
dropdown-network-forget = Forget
dropdown-network-dismiss = Dismiss
dropdown-network-error = Error
dropdown-network-cancel = Cancel
dropdown-network-password-placeholder = Enter password
dropdown-network-ethernet = Ethernet
dropdown-network-wifi = WiFi
dropdown-network-no-networks-title = No Networks Found
dropdown-network-no-networks-description = Make sure WiFi is enabled and try scanning again
dropdown-network-no-adapter-title = No WiFi Adapter
dropdown-network-no-adapter-description = No wireless adapter was detected on this system

## Security Types

dropdown-network-security-open = Open
dropdown-network-security-wep = WEP
dropdown-network-security-wpa = WPA
dropdown-network-security-wpa2 = WPA2
dropdown-network-security-wpa3 = WPA3
dropdown-network-security-enterprise = Enterprise
dropdown-network-security-saved = { $security } · Saved

## Connection Steps

dropdown-network-step-preparing = Preparing...
dropdown-network-step-configuring = Configuring...
dropdown-network-step-authenticating = Authenticating...
dropdown-network-step-obtaining-ip = Obtaining IP address...
dropdown-network-step-verifying = Verifying connection...

## Connection Errors

dropdown-network-error-wrong-password = Authentication failed
dropdown-network-error-timeout = Connection timed out
dropdown-network-error-ip-config = Failed to obtain IP address
dropdown-network-error-not-found = Network not found
dropdown-network-error-generic = Connection failed

### Weather Dropdown

dropdown-weather-title = Weather

## Stats
dropdown-weather-humidity = Humidity
dropdown-weather-wind = Wind
dropdown-weather-uv = UV Index
dropdown-weather-rain = Rain

## Sections
dropdown-weather-hourly = Hourly Forecast
dropdown-weather-daily = 5-Day Forecast

## Sun Times
dropdown-weather-sunrise = Sunrise
dropdown-weather-sunset = Sunset

## Time Display
# { $minutes } is the number of minutes since last update
dropdown-weather-updated-ago = Updated { $minutes }m ago
dropdown-weather-today = Today
dropdown-weather-now = Now

## Day Abbreviations
dropdown-weather-day-sun = Sun
dropdown-weather-day-mon = Mon
dropdown-weather-day-tue = Tue
dropdown-weather-day-wed = Wed
dropdown-weather-day-thu = Thu
dropdown-weather-day-fri = Fri
dropdown-weather-day-sat = Sat

## Actions
dropdown-weather-refresh = Refresh

## States
dropdown-weather-loading = Fetching weather data...
dropdown-weather-error-title = Unable to Load Weather
dropdown-weather-error-api-key = { $provider } requires an API key.
dropdown-weather-error-location = Location "{ $query }" was not found.
dropdown-weather-error-network = Cannot reach the weather service.
dropdown-weather-error-rate-limit = Too many requests. Try again later.
dropdown-weather-error-unknown = Something went wrong. Try again later.
dropdown-weather-retry = Retry

