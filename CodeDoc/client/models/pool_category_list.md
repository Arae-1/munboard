---
title: PoolCategoryList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, pool, category, list]
sources: [pool_category_list.js]
---

# PoolCategoryList Model

The `PoolCategoryList` class extends `AbstractList`. It manages a list of pool categories and tracks the default category and deleted categories for batch save operations.

## Static Properties

- `_itemClass` = `PoolCategory`
- `_itemName` = `"poolCategory"`

## Constructor

Calls `super()`. Initializes:
- `_defaultCategory` to `null`.
- `_origDefaultCategory` to `null`.
- `_deletedCategories` as an empty array.

Registers a `"remove"` event listener that calls `_evtCategoryDeleted()`.

## Static Method: `fromResponse(response)`

Calls `super.fromResponse(response)`. Iterates through the list to find the category where `isDefault` is true and stores it as `_defaultCategory`. Also stores it as `_origDefaultCategory`.

## Static Method: `get()`

Fetches all pool categories from `GET /api/pool-categories`. Returns a promise that resolves with the response object where `results` is a `PoolCategoryList` instance.

## Getter and Setter: `defaultCategory`

Get: returns `_defaultCategory`.
Set: sets `_defaultCategory`.

## Method: `save()`

Performs batch save:
1. Collects save promises for each category in the list.
2. Collects delete promises for each category in `_deletedCategories`.
3. If the default category changed, adds a `PUT /api/pool-category/<name>/default` request.
4. Returns `Promise.all(promises)`. On resolution, clears `_deletedCategories`.

## Method: `_evtCategoryDeleted(e)`

If the removed category is not transient (it has an `_origName`), pushes it into `_deletedCategories` for later deletion during `save()`.
