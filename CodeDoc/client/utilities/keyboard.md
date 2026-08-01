---
title: Keyboard Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, keyboard, shortcuts, mousetrap]
sources: [keyboard.js]
---

# Keyboard Utility

The `keyboard.js` file wraps the `mousetrap` library for keyboard shortcut handling. It respects user settings and supports pausing shortcuts.

## Module-Level Variable

- `paused` — a boolean flag. When `true`, all keyboard shortcuts are suppressed. Initially `false`.

## Mousetrap Patch

The `mousetrap.prototype.stopCallback` is overridden. It calls the original `stopCallback` only if `paused` is `false`. When `paused` is `true`, it returns `true` (suppressing the shortcut).

## Function: `bind(hotkey, func)`

If `settings.get().keyboardShortcuts` is `true`, calls `mousetrap.bind(hotkey, func)` and returns `true`. Otherwise, does nothing and returns `false`.

## Function: `bindElement(element, hotkey, func)`

Same as `bind()`, but binds the hotkey to a specific DOM element using `mousetrap(element).bind(hotkey, func)`.

## Function: `unbind(hotkey)`

Calls `mousetrap.unbind(hotkey)` unconditionally.

## Method: `pause()`

Sets `paused = true`. All keyboard shortcuts stop responding.

## Method: `unpause()`

Sets `paused = false`. Keyboard shortcuts resume.

## Exports

```js
{ bind, bindElement, unbind, pause, unpause }
```
