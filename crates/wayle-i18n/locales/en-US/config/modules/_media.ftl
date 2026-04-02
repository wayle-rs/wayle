### Wayle Configuration - Media Module

## Media Module Configuration

settings-modules-media-icon-type = Icon Type
    .description = Icon display mode (default, application, spinning-disc, application-mapped). spinning-disc uses slightly more CPU due to animation

settings-modules-media-player-icons = Player Icons
    .description = Custom player-to-icon mappings (glob pattern to icon name)

settings-modules-media-players-ignored = Ignored Players
    .description = Player bus name patterns to exclude from discovery

settings-modules-media-player-priority = Player Priority
    .description = Preferred player order as glob patterns matching bus names

settings-modules-media-format = Format
    .description = Label format with placeholders: {"{{ title }}"}, {"{{ artist }}"}, {"{{ album }}"}, {"{{ status }}"}, {"{{ status_icon }}"}

settings-modules-media-icon-name = Icon Name
    .description = Symbolic icon name for default mode

settings-modules-media-spinning-disc-icon = Spinning Disc Icon
    .description = Icon shown for spinning-disc mode

settings-modules-media-border-show = Show Border
    .description = Display border around button

settings-modules-media-border-color = Border Color
    .description = Border color token

settings-modules-media-icon-show = Show Icon
    .description = Display module icon

settings-modules-media-icon-color = Icon Color
    .description = Icon foreground color

settings-modules-media-icon-bg-color = Icon Background
    .description = Icon container background color

settings-modules-media-label-show = Show Label
    .description = Display text label

settings-modules-media-label-color = Label Color
    .description = Label text color

settings-modules-media-label-max-length = Label Max Length
    .description = Max characters before truncation

settings-modules-media-button-bg-color = Button Background
    .description = Button background color

settings-modules-media-left-click = Left Click
    .description = Action on left click

settings-modules-media-right-click = Right Click
    .description = Shell command on right click

settings-modules-media-middle-click = Middle Click
    .description = Shell command on middle click

settings-modules-media-scroll-up = Scroll Up
    .description = Shell command on scroll up

settings-modules-media-scroll-down = Scroll Down
    .description = Shell command on scroll down


## MediaIconType variants
enum-media-icon-type-default = Default
enum-media-icon-type-application = Application
enum-media-icon-type-spinning-disc = Spinning Disc
enum-media-icon-type-application-mapped = Application Mapped
