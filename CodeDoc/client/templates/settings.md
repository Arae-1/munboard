---
title: Settings Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, settings]
---

# Settings Templates

## `settings.tpl`
**Used by**: SettingsView
**Variables**: `ctx.browsingSettings` (all settings), `ctx.formatClientLink`, `ctx.makeCheckbox`, `ctx.makeNumericInput`
**Structure**: Form with heading and description (local storage note). Settings list:
- Enable keyboard shortcuts (checkbox, with help link icon)
- Number of posts per page (numeric input, 10-100 range)
- Use dark theme (checkbox)
- Upscale small posts (checkbox)
- Enable endless scroll (checkbox, with hint text)
- Use post flow (checkbox, with hint text)
- Enable transparency grid (checkbox, with hint text)
- Show tag suggestions (checkbox, with hint text)
- Automatically play video posts (checkbox)
- Display underscores as spaces (checkbox, with hint text)
- Accent color (color input)

Messages and save button at bottom.
