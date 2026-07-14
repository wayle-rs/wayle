### Wayle Configuration - Hyprland Workspaces Module

## Hyprland Workspaces Module Configuration

settings-modules-hyprland-workspaces-min-workspace-count = Minimum Workspaces
    .description = Minimum buttons to show (0 = only active/occupied)

settings-modules-hyprland-workspaces-monitor-specific = Monitor Specific
    .description = Show only workspaces on this monitor

settings-modules-hyprland-workspaces-show-special = Show Special
    .description = Include scratchpad workspaces

settings-modules-hyprland-workspaces-urgent-show = Show Urgent
    .description = Pulse animation on workspaces with urgent windows

settings-modules-hyprland-workspaces-display-mode = Display Mode
    .description = What identifies each workspace (label, icon, or none)

settings-modules-hyprland-workspaces-label-use-name = Use Name
    .description = Show workspace name instead of number

settings-modules-hyprland-workspaces-numbering = Numbering
    .description = How workspace numbers are displayed (absolute or relative)

settings-modules-hyprland-workspaces-divider = Divider
    .description = Text between workspace identity and app icons

settings-modules-hyprland-workspaces-app-icons-show = Show App Icons
    .description = Display window icons per workspace

settings-modules-hyprland-workspaces-app-icons-dedupe = Deduplicate Icons
    .description = Show one icon per window class

settings-modules-hyprland-workspaces-app-icons-fallback = Fallback Icon
    .description = Icon for unrecognized applications

settings-modules-hyprland-workspaces-app-icons-empty = Empty Icon
    .description = Placeholder icon for empty workspaces

settings-modules-hyprland-workspaces-icon-gap = Icon Gap
    .description = Spacing between app icons (rem)

settings-modules-hyprland-workspaces-workspace-padding = Workspace Padding
    .description = Padding along bar direction (rem)

settings-modules-hyprland-workspaces-icon-size = Icon Size
    .description = Scale multiplier for workspace icons (0.25-3.0)

settings-modules-hyprland-workspaces-label-size = Label Size
    .description = Scale multiplier for workspace labels (0.25-3.0)

settings-modules-hyprland-workspaces-workspace-ignore = Ignore Workspaces
    .description = Glob patterns for workspace IDs to hide

settings-modules-hyprland-workspaces-active-indicator = Active Indicator
    .description = Visual style for the active workspace

settings-modules-hyprland-workspaces-active-color = Active Color
    .description = Color for focused workspace icons and labels

settings-modules-hyprland-workspaces-occupied-color = Occupied Color
    .description = Color for occupied workspace icons and labels

settings-modules-hyprland-workspaces-empty-color = Empty Color
    .description = Color for empty workspace icons and labels

settings-modules-hyprland-workspaces-container-bg-color = Container Background
    .description = Background color for the workspaces container

settings-modules-hyprland-workspaces-border-show = Show Border
    .description = Display border around the workspaces container

settings-modules-hyprland-workspaces-border-color = Border Color
    .description = Border color for the workspaces container

settings-modules-hyprland-workspaces-workspace-map = Workspace Map
    .description = Per-workspace icon and color overrides

settings-modules-hyprland-workspaces-app-icon-map = App Icon Map
    .description = Window class/title to icon mappings

settings-modules-hyprland-workspaces-urgent-mode = Urgent Mode
    .description = How urgent workspaces are highlighted (per-workspace or per-application)


## DisplayMode variants
enum-display-mode-label = Label
enum-display-mode-icon = Icon
enum-display-mode-none = None

## ActiveIndicator variants
enum-active-indicator-background = Background
enum-active-indicator-underline = Underline
