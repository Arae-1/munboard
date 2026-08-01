---
title: PoolCategory Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, pool, category]
sources: [pool_category.js]
---

# PoolCategory Model

The `PoolCategory` class represents a category for pools. Each category has a name, display color, and metadata. It extends `EventTarget`.

## Constructor

Initializes:
- `_name` to `""`.
- `_color` to `"#000000"`.
- `_poolCount` to `0`.
- `_isDefault` to `false`.
- `_origName` to `null`.
- `_origColor` to `null`.

## Properties (Getters)

| Getter       | Return Type | Description                                    |
|--------------|-------------|------------------------------------------------|
| `name`       | String      | Category name.                                 |
| `color`      | String      | Display color (hex).                           |
| `poolCount`  | Number      | Number of pools in this category.              |
| `isDefault`  | Boolean     | True if this is the default category.          |
| `isTransient`| Boolean     | True if `_origName` is null (new, unsaved).    |

## Properties (Setters)

| Setter  | Type   | Description                 |
|---------|--------|-----------------------------|
| `name`  | String | Sets the category name.     |
| `color` | String | Sets the display color.     |

## Static Method: `fromResponse(response)`

Creates a `PoolCategory` and populates it from an API response. Returns the instance.

## Method: `save()`

Builds a diff between current values and `_origName`/`_origColor`. Only sends changed fields:
- `name` if different from `_origName`.
- `color` if different from `_origColor`.

If no fields changed, resolves immediately.

- **If `_origName` exists**: sends `PUT /api/pool-category/<origName>`.
- **If `_origName` is falsy**: sends `POST /api/pool-categories`.

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete()`

Sends `DELETE /api/pool-category/<origName>` with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field           | Internal Property |
|---------------------|-------------------|
| `response.version`  | `_version`        |
| `response.name`     | `_name`           |
| `response.color`    | `_color`          |
| `response.default`  | `_isDefault`      |
| `response.usages`   | `_poolCount`      |
| current `_name`     | `_origName`       |
| current `_color`    | `_origColor`      |

## Events Emitted

| Event    | Detail                      |
|----------|-----------------------------|
| `change` | `{ poolCategory: this }`    |
| `delete` | `{ poolCategory: this }`    |
