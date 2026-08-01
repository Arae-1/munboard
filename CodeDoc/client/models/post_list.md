---
title: PostList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, post, list]
sources: [post_list.js]
---

# PostList Model

The `PostList` class extends `AbstractList`. It provides post search, ID-based lookup, and search query decoration with safety settings.

## Static Properties

- `_itemClass` = `Post`
- `_itemName` = `"post"`

## Static Methods

### `getAround(id, searchQuery, options)`

Fetches the IDs of posts around a given post within a search context. Calls `GET /api/post/<id>/around` with `query` (decorated via `_decorateSearchQuery`) and `fields=id`. Returns the raw response (not a `PostList`).

### `search(text, offset, limit, fields)`

Searches posts via `GET /api/posts` with `query` (decorated), `offset`, `limit`, and `fields` (joined by comma). Returns a promise resolving to the response where `results` is a `PostList` instance.

### `_decorateSearchQuery(text)`

Uses browsing settings from `settings.get()`. If `api.safetyEnabled()` is true, checks `browsingSettings.listPosts` for disabled safety ratings. Prepends a `-rating:safe,sketchy,...` clause to exclude those ratings from search results. Returns the trimmed query.

## Methods

### `hasPostId(testId)`

Returns `true` if any post in the list has the given ID.

### `addById(id)`

If the post is not already in the list, creates a minimal `Post` with `{ id: id }` and adds it.

### `removeById(testId)`

Finds and removes the post matching the given ID.
