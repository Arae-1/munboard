---
title: Post Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, posts]
---

# Post Styles

## `post-main-view.styl`
**Imports**: `colors`

**Purpose**: Styles the main post view with sidebar and content.

**Key classes**:
- `.post-view` — flex row layout, full width
  - `>.sidebar` — fixed width (21em), margin-right, line-height. Navigation buttons as flex-wrap articles (33% each), centered links with hover darken effect. Dark theme uses box-shadow inset on hover
  - `>.content` — flex 1
  - `.post-container` — overflow-x auto for content
  - `.description-container` — `$top-navigation-color` background, 0.6em padding
  - `#post-description` — resizable vertical textarea, min-height 200px
  - Responsive at ≤800px: flex-wrap, sidebar moves below content (order 2), content order 1, mobile controls order 3

- `.readonly-sidebar` — Details with icons (download, upload info, safety, zoom, social with score-container and fav-container), Relations (2em thumbnails), Tags (compact tags)

- `.edit-sidebar` — Expander sections, safety radio wrapper (flex), management links, source textarea, form width auto, submit button full width, labels block. Focus border on text inputs in dark theme.

---

## `post-detail-view.styl`
**Purpose**: Styles the post detail page (summary/merge tabs).

**Key classes**:
- `#post` — max-width 40em, full width. Tab navigations with icons. Post merge layout: left/right panels at 47% width each, floated. Post thumbnails (9em height, full width). Header label inline, text input 6em width.

---

## `post-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the post thumbnail grid.

**Key classes**:
- `.post-list ul` — flexbox wrap, negative margin for gutter
- `li` — flex-grow, min-width 10em, width 12vw, min-height 7.5em, height 9vw
- `.thumbnail-wrapper` — full size, white text, box-shadow. Type badge (bottom-left, hidden for images), Stats (bottom-right, score/heart/comment icons with margins)
- `.edit-overlay` — positioned top-left. `.tag-flipper` (± button, green/red background with +/- text), `.safety-flipper` (colored circles with border and opacity for inactive), `.delete-flipper` (trash icon in red, `-` in gray when inactive)
- `.thumbnail` — full size, background-position 50% 30%
- `.no-tags` — red outline (4px), `.flagged` — orange outline (4px)
- Hover: background `$post-thumbnail-border-color`, thumbnail opacity .9, accent outline on focus/active
- `.post-flow` variant — content-aware sizing, 14vw height, 2px outlines, no fixed min-width
- `.post-list-header` — search text input 25em width (100% on mobile), safety toggle buttons with disabled transparency, bulk-edit forms with open/close/hidden states
- `.safety` buttons — colored backgrounds (safe/sketchy/unsafe) with `.disabled` at 15% opacity

---

## `post-content-control.styl`
**Imports**: `colors`

**Purpose**: Styles the post content display area.

**Key classes**:
- `.post-container` — text-align center
- `.post-content` — relative positioned container
- `.resize-listener` — absolute positioned, full size (0,0,0,0)
- `.transparency-grid img` — checkerboard background pattern via CSS gradients
- `img` — image-orientation from-image (respects EXIF rotation)

---

## `post-notes-overlay-control.styl`
**Imports**: `colors`

**Purpose**: Styles the note annotation overlay on posts.

**Key classes**:
- `.post-overlay` — states: `ready-to-draw`, `drawing-rectangle`, `drawing-polygon` show dashed border overlay; `read-only` has no pointer events
- `.notes-overlay g` — SVG polygon with fill/stroke; editing/drawing states have thicker stroke; ellipses for control points; nearby ellipse hover highlight
- `.note-text` — absolute positioned, max-width 22.5em, hidden by default with wrapper (lemonchiffon background, black border, black text)

---

## `post-upload.styl`
**Imports**: `colors`

**Purpose**: Styles the upload page.

**Key classes**:
- `#post-upload form` — max-width 40em, centered. `.inactive` hides submit/skip-duplicates; `.uploading` hides submit but shows cancel
- `.dropper-container` — large file-dropper (150% font, 2em padding)
- `.cancel` — tomato color
- `.uploadables-container li` — clearfix, padding-left for thumbnail float (12em width, 8em height). Uploadable body with header (nav buttons, filename ellipsis), safety radios, options, messages, lookalikes list (smaller thumbnails 6em×4em)
- Move buttons — first-child `.move-up` and last-child `.move-down` are inactive-colored

---

## Template: `index.htm`
**Not a `.tpl` file** — the main HTML entry point.

**Structure**: Standard HTML5 doctype with meta tags (viewport, theme-color, apple-mobile-web-app, msapplication), base tag placeholder (replaced at build time), app.min.css and vendor.min.css links, favicon/apple-touch-icon/splash screen links, manifest.json link. Body contains `#top-navigation-holder` and `#content-holder` divs, then loads vendor.min.js, app.min.js, and Ruffle (Flash emulator) asynchronously.
