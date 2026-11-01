---
title: general
outline: [2, 3]
---

# general

<div v-pre>

Shell-wide settings that don't belong to any specific module.

## General

| Field | Type | Default | Description |
|---|---|---|---|
| `font-sans` | string | `"Inter"` | Sans-serif font family for UI text and labels. |
| `font-mono` | string | `"JetBrains Mono"` | Monospace font family for code and technical content. |
| `tearing-mode` | bool | `false` | Demote overlay surfaces to allow compositor screen tearing. |

::: details More about `tearing-mode`

When enabled, surfaces that would normally use the `overlay` layer
are demoted to `top`, allowing fullscreen games to use direct scanout.

:::

## Default configuration

```toml
[general]
font-sans = "Inter"
font-mono = "JetBrains Mono"
tearing-mode = false
```


</div>
