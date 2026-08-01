---
title: Help View
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, help]
---

# Help View

## HelpView (`js/views/help_view.js`)

**Template**: `help` (main container)

**Purpose**: Renders help documentation with primary (section) and secondary (subsection) navigation.

**Section templates**:
- `about` → `help-about`
- `guidelines` → `help-guidelines`
- `keyboard` → `help-keyboard`
- `search` → `help-search`
- `comments` → `help-comments`
- `tos` → `help-tos`

**Subsection templates** (for `search` section):
- `default` → `help-search-general`
- `posts` → `help-search-posts`
- `users` → `help-search-users`
- `tags` → `help-search-tags`
- `pools` → `help-search-pools`

**Constructor** (`section`, `subsection`):
- Defaults section to `'about'`
- Defaults subsection to `'default'`
- Renders the main `help` template
- Replaces `.content` with the section template
- Replaces `.subcontent` with the subsection template (if applicable)
- Highlights active tab in both primary and secondary navigation
