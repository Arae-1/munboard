---
title: Home View
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, home]
---

# Home View

## HomeView (`js/views/home_view.js`)

**Templates**: `home`, `home-footer`, `home-featured-post`

**Purpose**: Landing page with site logo, search form, featured post display, and site statistics footer.

**Constructor** (`ctx`):
- `ctx.name` — site name
- `ctx.canListPosts` — permission flag
- Renders into `#content-holder`

**Render method**: Renders the `home` template. Creates a TagAutoCompleteControl on the search input if the form exists. Binds form submit to navigate to `/posts?query=<value>`.

**Key methods**:
- `setStats(stats)` — renders the footer with post count and disk usage
- `setFeaturedPost(postInfo)` — renders the featured post with PostContentControl and PostNotesOverlayControl

**Events**:
- Form submit navigates to post search page with query parameter
