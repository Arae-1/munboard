---
title: Note Model
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, models, note]
sources: [note.js]
---

# Note Model

The `Note` class represents an annotation note on a post. Each note has text and a polygon defined by points. It extends `EventTarget`.

## Constructor

Calls `super()`. Initializes:
- `_text` to `"…"` (default text).
- `_polygon` as a new `PointList` instance.

## Properties (Getters)

| Getter    | Return Type | Description                               |
|-----------|-------------|-------------------------------------------|
| `text`    | String      | The note's text content.                  |
| `polygon` | PointList   | The list of `Point` objects defining the polygon shape. |

## Properties (Setters)

| Setter  | Type   | Description                             |
|---------|--------|-----------------------------------------|
| `text`  | String | Sets the note's text content.           |

## Static Method: `fromResponse(response)`

Creates a `Note` and populates it from an API response object. Returns the note.

## Method: `_updateFromResponse(response)`

- Sets `_text` from `response.text`.
- Clears the current polygon.
- For each point in `response.polygon` (an array of `[x, y]` arrays), creates a new `Point(x, y)` and adds it to `_polygon`.

## Notes

- The polygon is an array of 2D point coordinates that outline the annotation area on an image post.
- The note does not have `save()` or `delete()` methods. It is saved as part of the parent `Post.save()` call (serialized in the post's `save()` method).
