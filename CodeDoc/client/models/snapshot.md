---
title: Snapshot Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, snapshot, history]
sources: [snapshot.js]
---

# Snapshot Model

The `Snapshot` class represents a historical record of a resource change (a version snapshot). It extends `EventTarget`. Snapshots are read-only and do not have `save()` or `delete()` methods.

## Constructor

Calls `super()`. Initializes `_orig` as an empty object and calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter      | Return Type | Description                                  |
|-------------|-------------|----------------------------------------------|
| `operation` | String      | The type of operation (e.g., "created", "modified"). |
| `type`      | String      | The resource type (e.g., "tag", "post").     |
| `id`        | any         | The resource ID.                             |
| `user`      | String      | The user who made the change.                |
| `data`      | Object      | The snapshot data (resource state at that time). |
| `time`      | String      | ISO timestamp of the snapshot.               |

## Static Method: `fromResponse(response)`

Creates a `Snapshot` and populates it from an API response. Returns the instance.

## Method: `_updateFromResponse(response)`

Maps API response fields to internal properties:

| API Field            | Internal Property |
|----------------------|-------------------|
| `response.operation` | `_operation`      |
| `response.type`      | `_type`           |
| `response.id`        | `_id`             |
| `response.user`      | `_user`           |
| `response.data`      | `_data`           |
| `response.time`      | `_time`           |

Values are copied via `Object.assign(this, map)`.
