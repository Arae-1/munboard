---
title: Settings and Common Component Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, common]
---

# Common Component Styles

## `pager.styl`
**Imports**: `colors`

**Purpose**: Styles the pager (manual and endless).

**Key classes**:
- `.pager nav .disabled` — opacity 0.5
- `.page` — relative positioned. `.page-header` before pseudo-element creates a horizontal line across the page, with the `span` overlaid on top with background matching window color
- Dark theme: line and span backgrounds use dark theme colors

---

## `expander-control.styl`
**Imports**: `colors`

**Purpose**: Styles the collapsible expander sections.

**Key classes**:
- `.expander.collapsed` — hides all children except header, margin-bottom 1em
- `header` — active tab background, line-height 2em. Link full block, color mix, font-size 120%. Chevron icon floated right
- `.expander-content` — padding 0.5em 0.5em 2em 0.5em
- Dark theme: header background and text color adjusted

---

## `nprogress.styl`
**Imports**: `colors`

**Purpose**: Styles the NProgress loading bar.

**Key classes**:
- `#nprogress` — pointer-events none
- `.bar` — accent color background, fixed top, full width, 2px height, z-index 1031
- `.spinner-icon`, `.peg` — hidden
- `.nprogress-custom-parent` — overflow hidden, relative position

---

## `home-view.styl`
**Imports**: (none)

**Purpose**: Styles the home page.

**Key classes**:
- `#home` — centered text, max-width 100%
- `.home-header` — flex, centered, gap 16px. `.home-logo` 3em height
- `header h1` — 2.5em, no margin
- `.messages` — centered with margin
- `form` — inline-block, auto width. `.sep` with margins. Inputs auto width, 2em height. At ≤500px: hide sep and links
- `.post-container` — flex centered, margin-bottom 2em, empty no margin
- `aside` — 90% font, no-wrap
- `footer` — 80% font, ul centered, li inline. `.sep` uses inline SVG circle as dot separator. Responsive at ≤800px: li block
- Dark theme: footer sep SVG uses light fill color

---

## `help-view.styl`
**Imports**: `colors`

**Purpose**: Styles the help pages.

**Key classes**:
- `#help` — max-width 55em, full width
- Navigation margin-bottom
- `td`, `th` — padding, first-child no-wrap
- `.section` — top margin 2em
- `h1` — top margin 2.5em, first-child 0
- `nav.secondary` — smaller font
- Responsive at ≤600px: hide table headers, convert to stacked rows, pre-wrap code blocks
