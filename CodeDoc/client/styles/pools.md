---
title: Pool Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, pools]
---

# Pool Styles

## `pool-view.styl`
**Purpose**: Styles the pool detail page.

**Key classes**:
- `#pool` — max-width 40em, full width. Heading word-break. Form full width.
- `.pool-edit textarea` — 10em height
- `.pool-summary` — Same structure as tag-summary: description top margin, details with inline comma-separated lists and `(none)` placeholder

---

## `pool-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the pool list table.

**Key classes**:
- `.pool-list table` — full width, row hover, header background
- Column widths: names 84%, post-count 8%, creation-time 8%
- Inline comma-separated name lists
- Dark theme variants
- Responsive at ≤800px: hide posts column

- `.pool-list-header` — Search input 25em width, append links

---

## `pool-input-control.styl`
**Imports**: `colors`

**Purpose**: Styles the pool input widget.

**Key classes**:
- `div.pool-input` — relative positioned
- `.main-control` — flex (input 5, button 1)
- `ul.compact-pools` — full width list, items with implication/new/duplicate colored backgrounds (references undefined color variables `$implied-pool-background-color` etc. — likely inherited from colors import)
- Pool usages/weight/remove links — inactive-link-color, unselectable

---

## `pool-categories-view.styl`
**Imports**: `colors`

**Purpose**: Styles the pool categories management page.

**Key classes**:
- `.content-wrapper.pool-categories` — max-width 45em
- `table` — border-spacing 0, default row background from `$default-pool-category-background-color` (undefined variable)
- `.color input` — 8em width
- `.usages` — centered
- `form` — width auto
