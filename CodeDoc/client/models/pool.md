---
title: Pool Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, pool]
sources: [pool.js]
---

# Pool Model

The `Pool` class represents a pool (collection) of posts. It extends `EventTarget` and supports CRUD, merge, save-with-diff, and category coloring.

## Constructor

Calls `super()`. Initializes `_orig` as an empty snapshot object for diff tracking. For both `this` and `_orig`, initializes `_posts` as a new `PostList`. Calls `_updateFromResponse({})` to set defaults.

The `PostList` require is deferred inside the constructor to avoid circular dependencies.

## Properties (Getters)

| Getter           | Return Type | Description                                           |
|------------------|-------------|-------------------------------------------------------|
| `id`             | any         | Pool ID.                                              |
| `names`          | Array       | Array of pool name strings.                           |
| `searchableName` | String      | First name if non-numeric, otherwise the ID as string.|
| `category`       | String      | Pool category name.                                   |
| `description`    | String      | Pool description text.                                |
| `posts`          | PostList    | List of posts in this pool.                           |
| `postCount`      | Number      | Total post count.                                     |
| `creationTime`   | String      | ISO creation timestamp.                               |
| `lastEditTime`   | String      | ISO last-edit timestamp.                              |
| `launchDate`     | String      | Pool launch date string.                              |

## Properties (Setters)

| Setter        | Type   | Description                            |
|---------------|--------|----------------------------------------|
| `names`       | Array  | Sets pool names.                       |
| `category`    | String | Sets pool category.                    |
| `description` | String | Sets pool description.                 |
| `launchDate`  | String | Sets pool launch date.                 |

## Static Method: `fromResponse(response)`

Creates a `Pool` and populates it from an API response. Returns the pool.

## Static Method: `get(id)`

Fetches a single pool by ID from the API (`GET /api/pool/<id>`). Returns a promise that resolves with a `Pool` instance.

## Method: `save()`

Builds a diff between the current state and `_orig`. Only changed fields are sent to avoid privilege violations:

- `names` — compared via `misc.arraysDiffer(name, orig, true)`.
- `category` — direct comparison.
- `description` — direct comparison.
- `posts` — compared via `misc.arraysDiffer()`, serialized as an array of post IDs.
- `launchDate` — direct comparison.

- **If `_id` exists**: sends `PUT /api/pool/<id>`.
- **If `_id` is falsy**: sends `POST /api/pools`.

On success, calls `_updateFromResponse()` and dispatches a `"change"` event.

## Method: `merge(targetId, addAlias)`

1. Fetches the target pool via `Pool.get(targetId)`.
2. Sends `POST /api/pool-merge` with `removeVersion`, `remove` (this pool's ID), `mergeToVersion`, and `mergeTo`.
3. On success, updates from the response and dispatches `"change"`.

## Method: `delete()`

Sends `DELETE /api/pool/<id>` with `version`. On success, dispatches a `"delete"` event.

## Method: `_updateFromResponse(response)`

Maps API response fields to internal properties:

| API Field               | Internal Property |
|-------------------------|-------------------|
| `response.id`           | `_id`             |
| `response.version`      | `_version`        |
| `response.names[0]`     | `_origName`       |
| `response.names`        | `_names`          |
| `response.category`     | `_category`       |
| `response.description`  | `_description`    |
| `response.creationTime` | `_creationTime`   |
| `response.lastEditTime` | `_lastEditTime`   |
| `response.launchDate`   | `_launchDate`     |
| `response.postCount`    | `_postCount` (default 0) |

For both `this` and `_orig`, `_posts.sync(response.posts)` is called. Then `Object.assign()` copies the mapped values to both objects.

## Events Emitted

| Event    | Detail              |
|----------|---------------------|
| `change` | `{ pool: this }`    |
| `delete` | `{ pool: this }`    |
