---
title: Search Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, search, keyboard]
sources: [search.js]
---

# Search Utility

The `search.js` file provides a keyboard shortcut helper for focusing the search input.

## Function: `searchInputNodeFocusHelper(inputNode)`

Binds the `q` keyboard shortcut (via `keyboard.bind`) to focus the given search input element and move the cursor to the end of the input text.

- Prevents the default browser action for the `q` key.
- Sets the selection range to the end of the input value.

## Exports

```js
{ searchInputNodeFocusHelper }
```
