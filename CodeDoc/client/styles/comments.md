---
title: Comment Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, comments]
---

# Comment Styles

## `comment-control.styl`
**Imports**: `colors`

**Purpose**: Styles individual comment display and editing.

**Key classes**:
- `$comment-header-background-color` — `$top-navigation-color` / darktheme variant
- `$comment-border-color` — `#DDD`
- `.comments-container` — top margin 2em
- `.comment-container` — left padding 60px (for avatar)
- `.avatar` — floated left, negative margin-left for positioning, 40px thumbnail. Focus outline on link
- `nav:not(.active)`, `.tab:not(.active)` — hidden
- `.comment` — 1px border. Header with edit tabs (Write/Preview), readonly header (nickname, date, score, edit/delete actions). Triangle arrow via `:before`/`:after` pseudo-elements pointing to avatar. Score container with upvote/downvote/value
- `.body` — margin 1em, width auto. `.keep-height` relative positioned for textarea (absolute, full size), `.tab.edit` min-height 150px
- `.messages` — margin
- Dark theme: comment border dark, header background dark, edit tab active background dark (`$window-color-darktheme`), arrow colors adjusted

- `.comment-content` — word-break on paragraphs, list-style inside, special styles for SJIS art (monospace font, light background, pre-wrap) and spoilers (hidden text by matching background, reveal on hover with brackets)

## `comment-list-view.styl`
**Imports**: `colors`

**Purpose**: Styles the global comment list page.

**Key classes**:
- `.global-comment-list` — text-align left
- `>ul` — Reset list. `>li` top border (3px solid, first-child excluded), top margin/padding
- Responsive at ≤700px: Post thumbnails 50vw × 33vw, margin-bottom
- Responsive at ≥700px: left padding 13em for thumbnail float (12em × 8em)
- Post thumbnails — vertical-align top, margin-right 1em
- `.comments-container` — full width
- Dark theme: border color dark

## `comment-list-control.styl`
**Purpose**: Simple list styling.

**Key classes**:
- `.comments > ul` — reset list
- `> li` — margin-bottom 1em, last-child margin 0
