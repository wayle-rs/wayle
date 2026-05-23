### Wayle Configuration - Triad Workspaces Module

## Triad Workspaces Module Configuration

settings-modules-triad-workspaces-min-workspace-count = Minimum Workspaces
    .description = Always show existing workspaces up to this index, even when empty

settings-modules-triad-workspaces-monitor-specific = Monitor Specific
    .description = Show only workspaces on this monitor

settings-modules-triad-workspaces-display-mode = Display Mode
    .description = What identifies each workspace (label, icon, or none)

settings-modules-triad-workspaces-label-strategy = Label Strategy
    .description = How to compose the workspace label from name and index

settings-modules-triad-workspaces-divider = Divider
    .description = Text between workspace identity and app icons

settings-modules-triad-workspaces-urgent-show = Show Urgent
    .description = Pulse animation on workspaces with urgent windows

settings-modules-triad-workspaces-urgent-mode = Urgent Mode
    .description = Pulse the whole workspace or only the urgent app icon

settings-modules-triad-workspaces-app-icons-show = Show App Icons
    .description = Display window icons per workspace

settings-modules-triad-workspaces-app-icons-dedupe = Deduplicate Icons
    .description = Show one icon per app_id instead of one per window

settings-modules-triad-workspaces-app-icons-fallback = Fallback Icon
    .description = Icon for windows not matched by app-icon-map

settings-modules-triad-workspaces-app-icons-empty = Empty Icon
    .description = Icon shown when a workspace has no application windows

settings-modules-triad-workspaces-icon-gap = Icon Gap
    .description = Spacing between app icons

settings-modules-triad-workspaces-workspace-padding = Workspace Padding
    .description = Padding along the bar direction

settings-modules-triad-workspaces-icon-size = Icon Size
    .description = Scale multiplier for workspace icons (0.25-3.0)

settings-modules-triad-workspaces-label-size = Label Size
    .description = Scale multiplier for workspace labels (0.25-3.0)

settings-modules-triad-workspaces-workspace-ignore = Ignore Workspaces
    .description = Glob patterns matched against name, index, or id

settings-modules-triad-workspaces-active-indicator = Active Indicator
    .description = Visual style for the active workspace

settings-modules-triad-workspaces-active-color = Active Color
    .description = Color for the active workspace icon and label

settings-modules-triad-workspaces-occupied-color = Occupied Color
    .description = Color for occupied workspace icons and labels

settings-modules-triad-workspaces-empty-color = Empty Color
    .description = Color for empty workspace icons and labels

settings-modules-triad-workspaces-container-bg-color = Container Background
    .description = Background color for the workspaces container

settings-modules-triad-workspaces-border-show = Show Border
    .description = Display border around the workspaces container

settings-modules-triad-workspaces-border-color = Border Color
    .description = Border color for the workspaces container

settings-modules-triad-workspaces-workspace-map = Workspace Map
    .description = Per-workspace icon and color overrides, keyed by name or id

settings-modules-triad-workspaces-app-icon-map = App Icon Map
    .description = Window app_id or title to icon mappings

settings-modules-triad-workspaces-left-click = Left Click
    .description = Action on left click

settings-modules-triad-workspaces-middle-click = Middle Click
    .description = Action on middle click

settings-modules-triad-workspaces-right-click = Right Click
    .description = Action on right click

settings-modules-triad-workspaces-scroll-up = Scroll Up
    .description = Action on scroll up

settings-modules-triad-workspaces-scroll-down = Scroll Down
    .description = Action on scroll down


## LabelStrategy variants
enum-label-strategy-index = Index
enum-label-strategy-name-or-index = Name or Index
enum-label-strategy-name-only = Name Only
enum-label-strategy-index-and-name = Index and Name
