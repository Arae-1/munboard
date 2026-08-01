---
title: TagCategory Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, tag, category]
sources: [tag_category.js]
---

# TagCategory Model

The `TagCategory` class represents a category for tags. Each category has a name, display color, sort order, and metadata. It extends `EventTarget`.

## Constructor

Initializes:
- `_name` to `""`.
- `_color` to `"#000000"`.
- `_order` to `1`.
- `_tagCount` to `0`.
- `_isDefault` to `false`.
- `_origName` to `null`.
- `_origColor` to `null`.
- `_origOrder` to `null`.

## Properties (Getters)

| Getter       | Return Type | Description                                    |
|--------------|-------------|------------------------------------------------|
| `name`       | String      | Category name.                                 |
| `color`      | String      | Display color (hex).                           |
| `order`      | Number      | Sort order (integer).                          |
| `tagCount`   | Number      | Number of tags in this category.               |
| `isDefault`  | Boolean     | True if this is the default category.          |
| `isTransient`| Boolean     | True if `_origName` is null (new, unsaved).    |

## Properties (Setters)

| Setter  | Type   | Description                              |
|---------|--------|------------------------------------------|
| `name`  | String | Sets the category name.                  |
| `color` | String | Sets the display color.                  |
| `order` | Number | Sets the sort order (parsed as integer). |

## Static Method: `fromResponse(response)`

Creates a `TagCategory` and populates it from an API response. Returns the instance.

## Method: `save()`

Builds a diff between current values and `_origName`/`_origColor`/`_origOrder`. Only sends changed fields:
- `name`, `color`, `order`.

If no fields changed, resolves immediately.

**If `_origName` exists**: `PUT /api/tag-category/<origName>`.
**If `_origName` is falsy**: `POST /api/tag-categories`.

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `delete()`

Sends `DELETE /api/tag-category/<origName>` with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field           | Internal Property |
|---------------------|-------------------|
| `response.version`  | `_version`        |
| `response.name`     | `_name`           |
| `response.color`    | `_color`          |
| `response.order`    | `_order`          |
| `response.default`  | `_isDefault`      |
| `response.usages`   | `_tagCount`       |
| current `_name`     | `_origName`       |
| current `_color`    | `_origColor`      |
| current `_order`    | `_origOrder`      |

## Events Emitted

| Event    | Detail                      |
|----------|-----------------------------|
| `change` | `{ tagCategory: this }`     |
| `delete` | `{ tagCategory: this }`     |
