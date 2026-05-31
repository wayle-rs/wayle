---
title: mango-workspaces
outline: [2, 3]
---

# mango-workspaces

<div v-pre>

MangoWM tag switcher module configuration.

Add it to your layout with `mango-workspaces`:

```toml
[[bar.layout]]
monitor = "*"
right = ["mango-workspaces"]
```

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `hide-empty` | bool | `true` | Hide tags that hold no clients and are not active. |
| `min-tag-count` | u8 | `0` | Always show tags up to this one-based index, even when empty. |
| `display-mode` | [`DisplayMode`](/config/types#display-mode) | `"label"` | What identifies each tag: its label, an icon, or nothing. |
| `divider` | string | `" "` | Text shown between the tag label and its application icons. |
| `app-icons-show` | bool | `false` | Show an application icon per client on each tag. |
| `app-icons-dedupe` | bool | `true` | Collapse clients that share an application to a single icon. |
| `app-icons-fallback` | string | `"ld-app-window-symbolic"` | Icon for clients not matched by `app-icon-map`. |
| `app-icons-empty` | string | `"tb-minus-symbolic"` | Icon shown when a tag has no clients. |
| `urgent-show` | bool | `true` | Highlight tags whose clients requested attention. |
| `urgent-mode` | [`UrgentMode`](/config/types#urgent-mode) | `"workspace"` | Whether urgency is tracked per tag or per application. |
| `active-indicator` | [`ActiveIndicator`](/config/types#active-indicator) | `"background"` | How the active tag is marked. |
| `tag-padding` | [`Spacing`](/config/types#spacing) | `0.5` | Padding around each tag button, in rem. |
| `icon-gap` | [`Spacing`](/config/types#spacing) | `0.3` | Spacing between application icons, in rem. |
| `icon-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale factor applied to application icons. |
| `label-size` | [`ScaleFactor`](/config/types#scale-factor) | `1` | Scale factor applied to the tag label text. |
| `active-color` | [`ColorValue`](/config/types#color-value) | `"accent"` | Color of the active tag. |
| `occupied-color` | [`ColorValue`](/config/types#color-value) | `"fg-muted"` | Color of tags that hold clients but are not active. |
| `empty-color` | [`ColorValue`](/config/types#color-value) | `"fg-subtle"` | Color of empty tags. |
| `container-bg-color` | [`ColorValue`](/config/types#color-value) | `"bg-surface-elevated"` | Background color of the tag container. |
| `border-show` | bool | `false` | Draw a border around the tag container. |
| `border-color` | [`ColorValue`](/config/types#color-value) | `"border-default"` | Border color when the border is shown. |
| `app-icon-map` | map of string | `{}` | Window-to-icon mappings for the application icons. |
| `tag-map` | map of [`WorkspaceStyle`](/config/types#workspace-style) | `{}` | Per-tag icon and color overrides, keyed by one-based tag index. |
| `left-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:this"` | Action for a left click on a tag. |
| `middle-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `""` | Action for a middle click on a tag. |
| `right-click` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `""` | Action for a right click on a tag. |
| `scroll-up` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:previous"` | Action for scrolling up over the tag container. |
| `scroll-down` | [`WorkspaceClickAction`](/config/types#workspace-click-action) | `"focus:next"` | Action for scrolling down over the tag container. |

::: details More about `min-tag-count`

`0` shows only occupied or active tags (subject to `hide-empty`). A
value above the compositor's tag count just shows every tag.

:::

::: details More about `app-icon-map`

Keys are glob patterns matched against a client's app id, or `title:`
patterns matched against its title. Values are symbolic icon names.

#### Example

```toml
[modules.mango-workspaces.app-icon-map]
"*firefox*" = "ld-globe-symbolic"
"title:*YouTube*" = "si-youtube-symbolic"
```

:::

::: details More about `tag-map`

#### Example

```toml
[modules.mango-workspaces.tag-map.1]
label = "web"
icon = "ld-globe-symbolic"
color = "#4a90d9"

[modules.mango-workspaces.tag-map.2]
label = "term"
icon = "ld-terminal-symbolic"
```

:::

## Default configuration

```toml
[modules.mango-workspaces]
hide-empty = true
min-tag-count = 0
display-mode = "label"
divider = " "
app-icons-show = false
app-icons-dedupe = true
app-icons-fallback = "ld-app-window-symbolic"
app-icons-empty = "tb-minus-symbolic"
urgent-show = true
urgent-mode = "workspace"
active-indicator = "background"
tag-padding = 0.5
icon-gap = 0.30000001192092896
icon-size = 1.0
label-size = 1.0
active-color = "accent"
occupied-color = "fg-muted"
empty-color = "fg-subtle"
container-bg-color = "bg-surface-elevated"
border-show = false
border-color = "border-default"
left-click = "focus:this"
middle-click = ""
right-click = ""
scroll-up = "focus:previous"
scroll-down = "focus:next"

[modules.mango-workspaces.app-icon-map]

[modules.mango-workspaces.tag-map]
```


</div>
