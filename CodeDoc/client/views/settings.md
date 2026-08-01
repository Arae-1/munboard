---
title: Settings View
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, settings]
---

# Settings View

## SettingsView (`js/views/settings_view.js`)

**Template**: `settings`

**Purpose**: User browsing settings form saved to browser local storage (not server-side).

**Constructor** (`ctx`):
- `ctx.settings` — the current BrowsingSettings object

**Render method**: Renders into `#content-holder`. Decorates validator on form.

**Settings fields** (all are checkboxes except where noted):
- `keyboard-shortcuts` — enable keyboard shortcuts (with help link)
- `posts-per-page` — numeric input, range 10-100
- `dark-theme` — use dark theme
- `upscale-small-posts` — upscale small posts
- `endless-scroll` — enable endless scroll instead of paged nav
- `post-flow` — use content-aware flow for post thumbnails
- `transparency-grid` — checkered pattern behind transparent posts
- `tag-suggestions` — show suggested tags popup in edit forms
- `autoplay-videos` — auto-play video posts (also sets `window.RufflePlayer.config.autoplay`)
- `underscores-as-spaces` — display underscores as spaces (visual only)
- `accent-color` — color picker input

**Events dispatched**:
- `submit` — with all settings values as an object
