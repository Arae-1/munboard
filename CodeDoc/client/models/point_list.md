---
title: PointList Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, point, list]
sources: [point_list.js]
---

# PointList Model

The `PointList` class extends `AbstractList`. It adds convenience getters for polygon vertex navigation.

## Static Properties

- `_itemClass` = `Point`
- `_itemName` = `"point"`

## Additional Getters

| Getter             | Description                                      |
|--------------------|--------------------------------------------------|
| `firstPoint`       | Returns `this._list[0]`, the first point.        |
| `secondLastPoint`  | Returns `this._list[this._list.length - 2]`.     |
| `lastPoint`        | Returns `this._list[this._list.length - 1]`.     |

## Usage

Used by `Note` to store the polygon's vertex points. The getters help with polygon editing UI logic (e.g., connecting new points to the last point).
