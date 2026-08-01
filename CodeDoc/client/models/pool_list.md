---
title: PoolList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, pool, list]
sources: [pool_list.js]
---

# PoolList Model

The `PoolList` class extends `AbstractList`. It adds search functionality and pool-ID-based operations for finding and removing pools from the list. The file is at `client/js/models/pool_list.js`.

## Static Properties

- `_itemClass` = `Pool` — Each list item is a `Pool` instance.
- `_itemName` = `"pool"` — The resource name for API calls.

## Static Method: `search(text, offset, limit, fields, options)`

Searches pools through the API. Calls `uri.formatApiLink("pools", ...)` to construct the URL `/api/pools`. Passes `query` (the search text), `offset`, `limit`, and `fields` (joined by comma) as query parameters. Uses `api.get()` with the given options.

On success, the response object contains `results` (raw API data). The method wraps `results` in a new `PoolList` instance via `PoolList.fromResponse()`. Returns a promise that resolves with the full response object where `results` is the hydrated list.

Parameters:
- `text` — Search query string.
- `offset` — Pagination offset.
- `limit` — Maximum results to return.
- `fields` — Array of field names. Joined with commas.
- `options` — Optional request configuration.

## Method: `hasPoolId(poolId)`

Checks if a pool with the given numeric ID exists in the list. Iterates through `this._list`. For each pool, compares `pool.id` to `poolId` using strict equality. Returns `true` on the first match. Returns `false` if no pool has the specified ID.

## Method: `removeById(poolId)`

Removes all pools from the list that have the given numeric ID. Iterates through `this._list`. For each pool, if `pool.id` equals `poolId`, calls `this.remove(pool)` to remove it from the internal `_list` array.

The method does not return a value. It operates synchronously. If multiple pools share the same ID (unlikely but possible), all are removed.

## Inheritance

Extends `AbstractList` which provides:
- `_list` — Internal array of items.
- `add(item)` — Adds an item to the list.
- `remove(item)` — Removes an item from the list by reference.
- `fromResponse(results)` — Creates a `PoolList` from raw API data. Iterates results and instantiates a `Pool` for each entry.
