---
title: Touch Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, touch, swipe, gesture]
sources: [touch.js]
---

# Touch Utility

The `touch.js` file provides touch swipe gesture detection. It defines swipe direction constants, handler functions, and a `Touch` class.

## Direction Constants

The `direction` object defines swipe direction values:

| Key    | Value     |
|--------|-----------|
| `NONE` | `null`    |
| `LEFT` | `"left"`  |
| `RIGHT`| `"right"` |
| `DOWN` | `"down"`  |
| `UP`   | `"up"`    |

## Function: `handleTouchStart(handler, evt)`

Records the initial touch position (`_xStart`, `_yStart`) from the first touch point. Also records the scroll edge state of the element (whether it is at the left or right scroll boundary).

## Function: `handleTouchMove(handler, evt)`

Calculates the difference between the start position and the current touch position. Sets `handler._direction` to the dominant direction (horizontal or vertical).

## Function: `handleTouchEnd(handler)`

Based on the detected direction, calls the corresponding task callback:

- `LEFT` — calls `_swipeLeftTask()` if at the right scroll edge.
- `RIGHT` — calls `_swipeRightTask()` if at the left scroll edge.
- `DOWN` — calls `_swipeDownTask()`.
- `UP` — calls `_swipeUpTask()`.

Clears `_xStart` and `_yStart` after handling.

## Class: Touch

### Constructor: `Touch(target, swipeLeft, swipeRight, swipeUp, swipeDown, getScrollElement)`

| Parameter          | Default            | Description                                   |
|--------------------|--------------------|-----------------------------------------------|
| `target`           | (required)         | The DOM element to attach touch listeners to. |
| `swipeLeft`        | `() => {}`         | Callback for a left swipe.                    |
| `swipeRight`       | `() => {}`         | Callback for a right swipe.                   |
| `swipeUp`          | `() => {}`         | Callback for an upward swipe.                 |
| `swipeDown`        | `() => {}`         | Callback for a downward swipe.                |
| `getScrollElement` | `() => target`     | Returns the scrollable element for edge detection. |

Registers `touchstart`, `touchmove`, and `touchend` event listeners on the target element.

### Instance Properties

- `_target` — the bound DOM element.
- `_getScrollElement` — function returning the scrollable element.
- `_swipeLeftTask`, `_swipeRightTask`, `_swipeUpTask`, `_swipeDownTask` — callback functions.
- `_xStart`, `_yStart` — initial touch coordinates.
- `_direction` — current swipe direction.
- `_atLeftEdge`, `_atRightEdge` — scroll boundary flags.

## Exports

The `Touch` class is the default export.
