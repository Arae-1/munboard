---
title: Snapshot Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, snapshots]
---

# Snapshot Views

## SnapshotsPageView (`js/views/snapshots_page_view.js`)

**Template**: `snapshots-page`

**Purpose**: Renders a list of history snapshots showing created, modified, deleted, and merged operations on posts, tags, tag categories, and pools.

**Constructor** (`ctx`):
- `ctx.response.results` — array of snapshot items
- `ctx.hostNode` — node to render into

**Render method**: Passes three helper functions to the template:
- `makeResourceLink(type, id, data)` — creates links for post/tag/tag_category/pool
- `makeItemCreation(type, data)` — formats item creation details (capitalizes keys, joins array values with commas)
- `makeItemModification(type, data)` — formats modification diffs for each resource type

All diff values are user-controlled and enter the DOM as HTML. The view escapes every diff value with `misc.escapeHtml` before inserting it. This applies to added/removed list items, old/new primitive values, tag category ids, and item creation keys and values.

**Modification formatting by type**:
- **tag_category**: name, color, default status changes
- **tag**: names, category, description, suggestions, implications changes
- **post**: checksum (content), featured, source, description, safety, tags, relations, notes, flags changes
- **pool**: names, category, description, posts changes

Difference types are `list change` (added/removed items), `primitive change` (old-value → new-value), or generic change.
