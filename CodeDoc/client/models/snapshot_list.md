---
title: SnapshotList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, snapshot, list, history]
sources: [snapshot_list.js]
---

# SnapshotList Model

The `SnapshotList` class extends `AbstractList`. It adds search functionality for historical snapshots.

## Static Properties

- `_itemClass` = `Snapshot`
- `_itemName` = `"snapshot"`

## Static Method: `search(text, offset, limit)`

Searches snapshots via `GET /api/snapshots` with parameters `query`, `offset`, and `limit`. Returns a promise resolving to the response object where `results` is a `SnapshotList` instance.
