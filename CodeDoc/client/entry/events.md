---
title: Events Module
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, events, event-target, proxy]
sources: [events.js]
---

# Events Module

The `events.js` file provides a custom event system for the client application. It defines an `EventTarget` base class and an event proxy function, plus message type constants.

## Class: EventTarget

A lightweight implementation of the DOM `EventTarget` interface using a `DocumentFragment` as the backing event target.

### Constructor

Creates a `DocumentFragment` via `document.createDocumentFragment()`. Binds three methods from the fragment to `this`:

- `addEventListener`
- `dispatchEvent`
- `removeEventListener`

This allows any class extending `EventTarget` to use the standard DOM event API without inheritance from `window.EventTarget`.

### Usage Pattern

All model classes (`Tag`, `Post`, `Pool`, `Comment`, `User`, etc.) extend `EventTarget`. They call `this.addEventListener("delete", handler)` to listen and `this.dispatchEvent(new CustomEvent("delete", { detail: ... }))` to emit events.

## Function: `proxyEvent(source, target, sourceEventType, targetEventType)`

Forwards events from one event target to another.

**Parameters:**
- `source` — the source event target.
- `target` — the destination event target.
- `sourceEventType` — the event type to listen for on the source.
- `targetEventType` — the event type to dispatch on the target (defaults to `sourceEventType`).

If `source.addEventListener` does not exist, the function returns without doing anything.

## Constants

- `Success` — the string `"success"`. Used as a CSS class for success messages.
- `Error` — the string `"error"`. Used as a CSS class for error messages.
- `Info` — the string `"info"`. Used as a CSS class for informational messages.

## Exports

```js
{
    Success: "success",
    Error: "error",
    Info: "info",
    proxyEvent,
    EventTarget,
}
```
