---
title: Point Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, point, coordinate]
sources: [point.js]
---

# Point Model

The `Point` class represents a 2D coordinate point. It extends `EventTarget` and emits `"change"` events when coordinates are modified.

## Constructor: `Point(x, y)`

- Calls `super()`.
- Sets `_x` to `x`.
- Sets `_y` to `y`.

## Properties (Getters and Setters)

### `x` (Number)

Get: returns `this._x`.
Set: sets `this._x`, dispatches a `"change"` event with `{ point: this }`.

### `y` (Number)

Get: returns `this._y`.
Set: sets `this._y`, dispatches a `"change"` event with `{ point: this }`.

## Events Emitted

| Event    | Detail              | When                  |
|----------|---------------------|-----------------------|
| `change` | `{ point: this }`   | When x or y is set.   |

## Usage

`Point` objects are created inside `Note._updateFromResponse()` from pairs of coordinates in the polygon array. Each `Point` represents a vertex of the note polygon on a post image.
