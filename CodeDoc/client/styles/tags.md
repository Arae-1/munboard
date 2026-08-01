---
title: Tag Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, tags]
---

# Tag Styles

## `tag-view.styl`
**Purpose**: Styles the tag detail page.

**Key classes**:
- `#tag` — max-width 40em, full width. Heading with word-break for long tag names. Form full width.
- `.tag-edit textarea` — 10em height
- `.tag-summary` — Description section top margin, Details section with inline comma-separated lists, `(none)` for empty lists, 1em margin between sections

---

## `tag-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the tag list table.

**Key classes**:
- `.tag-list table` — full width, row hover background, header with `$top-navigation-color` background
- Column widths: names 28%, implications 28%, suggestions 28%, usages 8%, creation-time 8%
- Inline comma-separated lists
- Dark theme row hover and header backgrounds
- Responsive at ≤800px: hide implications and suggestions columns

- `.tag-list-header` — Search input 25em width (100% on mobile), .append links with inactive-link-color

---

## `tag-input-control.styl`
**Imports**: `colors`

**Purpose**: Styles the tag input widget with auto-complete and suggestions panel.

**Key classes**:
- `div.tag-input` — relative positioned
- `.main-control` — flex layout (input flex 5, button flex 1 with left margin)
- `.tag-suggestions` — positioned left of input, hidden by default (`.shown` to display, `.translucent` for half opacity), arrow pointer via `:before` pseudo-element, wrapper with window-color background, border, 15em width, scrollable list max-height 20em, suggestion items with add-tag link (ellipsis, 10em max-width) and tag-weight
- `ul.compact-tags` — full width list, `.added`/`.new`/`.implication` with slide animation, colored backgrounds (green new, yellow implied, red duplicate), dark theme darkens these backgrounds
- Tag usages/weight/remove-tag — inactive-link-color, unselectable, small font

---

## `tag-categories-view.styl`
**Imports**: `colors`

**Purpose**: Styles the tag categories management page.

**Key classes**:
- `.content-wrapper.tag-categories` — max-width 45em
- `table` — border-spacing 0, default row background from `$default-tag-category-background-color`
- `.color input` — 8em width
- `.usages` — centered text
- `.remove`, `.set-default` — no-wrap
- `form` — width auto
