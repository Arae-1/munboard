---
title: User Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, users]
---

# User Styles

## `user-view.styl`
**Imports**: `colors`

**Purpose**: Styles the user detail page.

**Key classes**:
- `#user` — max-width 35em, full width
- `#user-summary` — thumbnail 6em square, floated left. Basic info list. Quick links nav (45% width, floated, margin-right 1em). Cleared bottom.
- `#user-edit` — form full width. Avatar section: radio buttons floated left (30%), file dropper floated right (65%), clearfix after.
- `#user-tokens` — Token flex container (column), token-flex-row (row, space-between). Labels column with right padding. Full-width class. hr with 3px solid $token-border-color. Form width 100%.
- `#user-delete form` — full width

---

## `user-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the user list page.

**Key classes**:
- `.user-list ul` — flexbox wrap. `li` flex-grow, 20em width, margin, padding 0.75em, `$top-navigation-color` background
- `.wrapper` — flex container
- `.image` — margin for avatar
- `.thumbnail` — 3em square
- Dark theme: li uses `$top-navigation-color-darktheme`

- `.user-list-header` — Search input 25em, append links

---

## `user-registration.styl`
**Imports**: `colors`

**Purpose**: Styles the registration page.

**Key classes**:
- `#user-registration` — extra bottom padding. Form floated left, margin-right 3em
- `.info` — floated left, 20em width, rounded corners. Icon color `$main-color`. List 1.8em line-height. First/last paragraph margins.

---

## `password-reset.styl`
**Purpose**: Minimal style — max-width 30em for `#password-reset`.
