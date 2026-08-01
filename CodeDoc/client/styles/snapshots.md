---
title: Snapshot Styles
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, styles, snapshots]
---

# Snapshot Styles

## `snapshots-list-view.styl`
**Purpose**: Styles the history/snapshot list.

**Key classes**:
- `.snapshot-list` — text-align left
- `ul` — max-width 35em, centered, list reset
- `li` — margin-bottom 1em, last-child 0
- `.time` — floated right
- `div` — padding 0.1em 0.5em, empty div has no padding

**Operation colors**:
- `operation-created` — `#E0F5E0` (green), details at 50% alpha
- `operation-modified` — `#E0F5FF` (blue), details at 50% alpha
- `operation-deleted` — `#FDE5E5` (red), details at 50% alpha
- `operation-merged` — `#FEC` (yellow), details at 50% alpha

**Dark theme**: All operation backgrounds darkened by 80%.
