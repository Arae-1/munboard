---
title: Optimized Resize Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, resize, performance]
sources: [optimized_resize.js]
---

# Optimized Resize Utility

The `optimized_resize.js` file provides a debounced window resize event handler. It batches resize callbacks using `requestAnimationFrame`.

## Variables

- `callbacks` — an array of registered callback functions.
- `running` — a boolean flag to prevent duplicate rAF requests.

## Function: `resize()`

Triggered by the `window.resize` event. If `running` is `false`, sets `running = true` and schedules `runCallbacks` via `requestAnimationFrame` (or `setTimeout` with 66ms as fallback).

## Function: `runCallbacks()`

Iterates through all registered callbacks and calls each one. Sets `running = false`.

## Function: `add(callback)`

Pushes a callback function onto the `callbacks` array.

## Function: `remove(callback)`

Filters out the given callback from the `callbacks` array.

## Module Load Behavior

At module load time, `window.addEventListener("resize", resize)` is called.

## Exports

```js
{ add, remove }
```
