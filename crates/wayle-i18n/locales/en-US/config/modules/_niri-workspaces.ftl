### Wayle Configuration - Niri Workspaces Module

## Niri Workspaces Module Configuration

settings-modules-niri-workspaces-min-workspace-count = Minimum Workspaces
    .description = Always show numerically-named workspaces up to this index, even when empty

settings-modules-niri-workspaces-monitor-specific = Monitor Specific
    .description = Show only workspaces on this monitor

settings-modules-niri-workspaces-hide-trailing-empty = Hide Trailing Empty
    .description = Hide niri's auto-allocated empty workspace at the end of each output

settings-modules-niri-workspaces-display-mode = Display Mode
    .description = What identifies each workspace (label, icon, or none)

settings-modules-niri-workspaces-label-strategy = Label Strategy
    .description = How to compose the workspace label from name and index

settings-modules-niri-workspaces-divider = Divider
    .description = Text between workspace identity and app icons

settings-modules-niri-workspaces-urgent-show = Show Urgent
    .description = Pulse animation on workspaces with urgent windows

settings-modules-niri-workspaces-urgent-mode = Urgent Mode
    .description = Pulse the whole workspace or only the urgent app icon

settings-modules-niri-workspaces-app-icons-show = Show App Icons
    .description = Display window icons per workspace

settings-modules-niri-workspaces-app-icons-dedupe = Deduplicate Icons
    .description = Show one icon per app_id instead of one per window

settings-modules-niri-workspaces-app-icons-fallback = Fallback Icon
    .description = Icon for windows not matched by app-icon-map

settings-modules-niri-workspaces-app-icons-empty = Empty Icon
    .description = Icon shown when a workspace has no application windows

settings-modules-niri-workspaces-icon-gap = Icon Gap
    .description = Spacing between app icons

settings-modules-niri-workspaces-workspace-padding = Workspace Padding
    .description = Padding along the bar direction

settings-modules-niri-workspaces-icon-size = Icon Size
    .description = Scale multiplier for workspace icons (0.25-3.0)

settings-modules-niri-workspaces-label-size = Label Size
    .description = Scale multiplier for workspace labels (0.25-3.0)

settings-modules-niri-workspaces-workspace-ignore = Ignore Workspaces
    .description = Glob patterns matched against name, index, or id

settings-modules-niri-workspaces-active-indicator = Active Indicator
    .description = Visual style for the active workspace

settings-modules-niri-workspaces-active-color = Active Color
    .description = Color for the active workspace icon and label

settings-modules-niri-workspaces-occupied-color = Occupied Color
    .description = Color for occupied workspace icons and labels

settings-modules-niri-workspaces-empty-color = Empty Color
    .description = Color for empty workspace icons and labels

settings-modules-niri-workspaces-container-bg-color = Container Background
    .description = Background color for the workspaces container

settings-modules-niri-workspaces-border-show = Show Border
    .description = Display border around the workspaces container

settings-modules-niri-workspaces-border-color = Border Color
    .description = Border color for the workspaces container

settings-modules-niri-workspaces-workspace-map = Workspace Map
    .description = Per-workspace icon and color overrides, keyed by name or id

settings-modules-niri-workspaces-app-icon-map = App Icon Map
    .description = Window app_id or title to icon mappings

settings-modules-niri-workspaces-left-click = Left Click
    .description = Action on left click

settings-modules-niri-workspaces-middle-click = Middle Click
    .description = Action on middle click

settings-modules-niri-workspaces-right-click = Right Click
    .description = Action on right click

settings-modules-niri-workspaces-scroll-up = Scroll Up
    .description = Action on scroll up

settings-modules-niri-workspaces-scroll-down = Scroll Down
    .description = Action on scroll down


## LabelStrategy variants
enum-label-strategy-index = Index
enum-label-strategy-name-or-index = Name or Index
enum-label-strategy-name-only = Name Only
enum-label-strategy-index-and-name = Index and Name
