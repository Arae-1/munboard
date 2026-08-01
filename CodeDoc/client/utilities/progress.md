---
title: Progress Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, progress, loading, nprogress]
sources: [progress.js]
---

# Progress Utility

The `progress.js` file provides a loading progress bar using the `nprogress` library. It supports nested request tracking.

## Module-Level Variable

- `nesting` — a counter tracking the number of active requests. Initialized to `0`.

## Function: `start()`

If `nesting` is `0`, calls `nprogress.start()` to show the progress bar. Increments `nesting` by 1.

## Function: `done()`

Decrements `nesting` by 1.
- If `nesting > 0`, calls `nprogress.inc()` to advance the bar slightly.
- If `nesting === 0`, calls `nprogress.done()` to complete and hide the bar.

## Usage

The `Api._rawRequest()` method calls `progress.start()` before each HTTP request and `progress.done()` when the request completes. The nesting counter prevents the progress bar from flickering during concurrent requests.

## Exports

```js
{ start, done }
```
