---
title: TagList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, tag, list]
sources: [tag_list.js]
---

# TagList Model

The `TagList` class extends `AbstractList`. It adds search, tag-name-based lookup, addition, removal, and recursive implication resolution. The file is at `client/js/models/tag_list.js`.

## Static Properties

- `_itemClass` = `Tag` — Each list item is a `Tag` instance.
- `_itemName` = `"tag"` — The resource name for API calls.

## Static Method: `search(text, offset, limit, fields, options)`

Searches tags through the API. Calls `uri.formatApiLink("tags", ...)` to construct the URL `/api/tags`. Passes `query` (the search text), `offset`, `limit`, and `fields` (joined by comma) as query parameters. Uses `api.get()` with the given options.

On success, the response object contains `results` (raw API data). The method wraps `results` in a new `TagList` instance via `TagList.fromResponse()`. Returns a promise that resolves with the full response object where `results` is the hydrated list.

Parameters:
- `text` — Search query string.
- `offset` — Pagination offset.
- `limit` — Maximum results to return.
- `fields` — Array of field names. Joined with commas.
- `options` — Optional request configuration.

## Method: `isTaggedWith(testName)`

Checks if any tag in the list has the given name. Iterates through `this._list`. For each tag, iterates through all `tag.names`. Performs a case-insensitive comparison. Returns `true` on the first match. Returns `false` if no tag matches.

## Method: `addByName(tagName, addImplications)`

Adds a tag to the list by name. The flow is:

1. Call `this.isTaggedWith(tagName)`. If the tag name already exists in the list, resolve the promise immediately (no-op).
2. Create a transient `Tag` instance. Set `tag.names = [tagName]`.
3. Call `this.add(tag)` to insert it into the internal `_list` array.
4. If `addImplications` is exactly `false`, resolve immediately. Do not fetch implications.
5. If `addImplications` is truthy (default behavior):
   - Call `Tag.get(tagName)` to fetch the full tag from the server.
   - The full tag includes `implications` — an array of related tags.
   - For each implication, call `this.addByName(relation.names[0], true)` recursively.
   - Use `Promise.all()` to wait for all recursive additions.

This method correctly handles chain implications. If tag A implies B and B implies C, adding A recursively adds B and then C. The second parameter controls recursion: it passes `true` to sub-calls and stops when the caller passes `false`.

## Method: `removeByName(testName)`

Removes any tag from the list that has the given name. Iterates through `this._list`. For each tag, iterates through all `tag.names`. Performs a case-insensitive comparison. If a match is found, calls `this.remove(tag)` to remove it from the internal `_list` array.

The method does not return a value. It operates synchronously. Multiple tags can match (if two tags share a name alias), and all are removed.

## Inheritance

Extends `AbstractList` which provides:
- `_list` — Internal array of items.
- `add(item)` — Adds an item.
- `remove(item)` — Removes an item.
- `fromResponse(results)` — Creates a `TagList` from raw API data.
