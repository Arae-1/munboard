---
title: Polyfill Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, polyfill, browser]
sources: [polyfill.js]
---

# Polyfill Utility

The `polyfill.js` file adds non-standard methods and fixes to native browser objects. It is loaded early in the startup sequence in `main.js`.

## NodeList Extensions

### `NodeList.prototype[Symbol.iterator]`

Sets the iterator to `Array.prototype[Symbol.iterator]`, making NodeLists iterable in Chrome and Opera.

### `NodeList.prototype.querySelector(...args)`

Iterates through the nodes in the list. Skips text nodes (nodeType 3). Returns the first matching element found by `querySelector()`, or `null`.

### `NodeList.prototype.querySelectorAll(...args)`

Iterates through all nodes (skipping text nodes). Collects all matching elements from each node's `querySelectorAll()` call. Returns the combined array.

## Node Extension

### `Node.prototype.prependChild(child)`

Inserts `child` as the first child of the node. If the node has no children, appends the child instead.

## Promise Extensions

### `Promise.prototype.always(onResolveOrReject)`

Adds a callback that executes on both resolve and reject, similar to jQuery's `.always()`. On rejection, re-throws the reason after the callback.

### `Promise.prototype.abort`

A no-op function. Provides a default `abort()` method on all promises. Specific promises (e.g., API request promises) override this with a functional abort.

## Number Extension

### `Number.prototype.between(a, b, inclusive)`

Returns `true` if the number is between `a` and `b`. If `inclusive` is true, includes the boundaries.

## Date Extension

### `Date.prototype.addDays(days)`

Returns a new `Date` object with the specified number of days added.

## Usage

All extensions are applied at module load time (when `require("./util/polyfill.js")` is called in `main.js`). They modify built-in prototypes globally.
