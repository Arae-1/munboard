---
title: Tag Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, tag]
sources: [tag.js]
---

# Tag Model

The `Tag` class represents a tag with multiple names, a category, description, suggestions, and implications. It extends `EventTarget` and supports CRUD, merge, and diff-based save.

## Constructor

Defers the `TagList` require to avoid circular dependencies. Calls `super()`. Initializes `_orig` as an empty object. For both `this` and `_orig`, initializes `_suggestions` and `_implications` as `TagList` instances. Calls `_updateFromResponse({})`.

## Properties (Getters)

| Getter         | Return Type | Description                                    |
|----------------|-------------|------------------------------------------------|
| `names`        | Array       | Array of tag name strings.                     |
| `category`     | String      | Tag category name.                             |
| `description`  | String      | Tag description text.                          |
| `suggestions`  | TagList     | Suggested subordinate tags.                    |
| `implications` | TagList     | Implied tags (automatically added with this tag).|
| `postCount`    | Number      | Number of posts using this tag.                |
| `creationTime` | String      | ISO creation timestamp.                        |
| `lastEditTime` | String      | ISO last-edit timestamp.                       |

## Properties (Setters)

| Setter        | Type   | Description                        |
|---------------|--------|------------------------------------|
| `names`       | Array  | Sets tag names.                    |
| `category`    | String | Sets tag category.                 |
| `description` | String | Sets tag description.              |

## Static Methods

### `fromResponse(response)`
Creates a `Tag` and populates it from an API response.

### `get(name)`
Fetches a single tag by name via `GET /api/tag/<name>`. Returns a promise resolving to a `Tag` instance.

## Method: `save()`

Builds a diff between current state and `_orig`. Only sends changed fields:
- `names` — compared via `misc.arraysDiffer(names, orig, true)`.
- `category`, `description` — direct comparison.
- `implications` — compared via `misc.arraysDiffer()`, serialized as `relation.names[0]`.
- `suggestions` — compared via `misc.arraysDiffer()`, serialized as `relation.names[0]`.

**If `_origName` exists**: `PUT /api/tag/<origName>`.
**If `_origName` is falsy**: `POST /api/tags`.

On success, calls `_updateFromResponse()` and dispatches `"change"`.

## Method: `merge(targetName, addAlias)`

1. Fetches the target tag via `Tag.get(targetName)`.
2. Sends `POST /api/tag-merge` with `removeVersion`, `remove` (orig name), `mergeToVersion`, `mergeTo`.
3. On success, updates from response and dispatches `"change"`.

## Method: `delete()`

Sends `DELETE /api/tag/<origName>` with `version`. On success, dispatches `"delete"`.

## Method: `_updateFromResponse(response)`

Maps API response fields:

| API Field               | Internal Property |
|-------------------------|-------------------|
| `response.version`      | `_version`        |
| `response.names[0]`     | `_origName`       |
| `response.names`        | `_names`          |
| `response.category`     | `_category`       |
| `response.description`  | `_description`    |
| `response.creationTime` | `_creationTime`   |
| `response.lastEditTime` | `_lastEditTime`   |
| `response.usages`       | `_postCount` (default 0)|

For both `this` and `_orig`, calls `_suggestions.sync(response.suggestions)` and `_implications.sync(response.implications)`. Then `Object.assign()` to both objects.

## Events Emitted

| Event    | Detail             |
|----------|--------------------|
| `change` | `{ tag: this }`    |
| `delete` | `{ tag: this }`    |
