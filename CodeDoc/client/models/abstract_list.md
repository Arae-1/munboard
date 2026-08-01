---
title: AbstractList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, list, base-class]
sources: [abstract_list.js]
---

# AbstractList Model

The `AbstractList` class is the base class for all list-type model classes. It extends `EventTarget` and maintains an internal array of items. It provides iteration, filtering, mapping, and observable add/remove operations.

## Static Properties (set by subclasses)

- `_itemClass` — the constructor for individual items in the list. Used by `fromResponse()`.
- `_itemName` — a string name for the item type (e.g., `"tag"`, `"comment"`). Used in event detail objects.

## Constructor

Initializes `_list` as an empty array. Calls `super()` to set up event handling.

## Static Method: `fromResponse(response)`

Creates a new instance of the subclass. Iterates through the `response` array. For each item:
1. Calls `this._itemClass.fromResponse(item)` to instantiate the item.
2. If the item has `addEventListener`, registers:
   - A `"delete"` listener that calls `ret.remove(addedItem)`.
   - A `"change"` listener that re-dispatches the `"change"` event from the item to the list.
3. Pushes the item into `ret._list`.

Returns the populated list instance.

## Method: `sync(plainList)`

Clears the list via `this.clear()`. For each item in `plainList`, creates a model instance via `this.constructor._itemClass.fromResponse()` and adds it via `this.add()`.

Used by models like `Post`, `Tag`, and `Pool` to synchronize embedded lists from API responses.

## Method: `add(item)`

- If the item has `addEventListener`, registers `"delete"` and `"change"` event listeners on the item.
- Pushes the item into `_list`.
- Dispatches an `"add"` event with the item in the detail object (keyed by `_itemName`).

## Method: `clear()`

Removes every item in the list by calling `this.remove()` for each item (using a shallow copy to avoid iteration issues).

## Method: `remove(itemToRemove)`

- Searches `_list` for the item using identity comparison (`!==`).
- If found, splices it from the array (index-based removal).
- Dispatches a `"remove"` event with the removed item in the detail object.

## Getter: `length`

Returns `this._list.length`.

## Method: `at(index)`

Returns the item at the given index in `_list`.

## Method: `map(...args)`

Delegates to `this._list.map(...args)`.

## Method: `filter(...args)`

Delegates to `this._list.filter(...args)`.

## Iterator: `[Symbol.iterator]()`

Makes the list iterable via `for...of` by delegating to `this._list[Symbol.iterator]()`.

## Events Emitted

| Event      | Detail Key                  | When                                     |
|------------|-----------------------------|------------------------------------------|
| `add`      | `[this._itemName]`          | After an item is added.                  |
| `remove`   | `[this._itemName]`          | After an item is removed.                |
| `change`   | forwarded from item detail  | When a contained item dispatches change. |

## Subclasses

- `CommentList` (item: `Comment`)
- `CustomFieldList` (item: `CustomField`)
- `NoteList` (item: `Note`)
- `PointList` (item: `Point`)
- `PoolList` (item: `Pool`)
- `PoolCategoryList` (item: `PoolCategory`)
- `PostList` (item: `Post`)
- `SnapshotList` (item: `Snapshot`)
- `TagList` (item: `Tag`)
- `TagCategoryList` (item: `TagCategory`)
- `UserList` (item: `User`)
