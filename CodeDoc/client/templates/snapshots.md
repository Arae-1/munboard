---
title: Snapshot Templates
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, templates, snapshots]
---

# Snapshot Templates

## `snapshots_page.tpl`
**Used by**: SnapshotsPageView
**Variables**: `ctx.response.results`, `ctx.makeRelativeTime`, `ctx.makeUserLink`, `ctx.makeResourceLink`, `ctx.makeItemCreation`, `ctx.makeItemModification`
**Structure**: List of snapshot items. Each item has:
- **Header**: relative time, user link, operation type (created/modified/merged), resource link
- **Details**: 
  - `created` — calls `makeItemCreation(type, data)` with the stored data
  - `modified` — calls `makeItemModification(type, data)` with diff data
  - `merged` — shows "Merged to" with resource link
