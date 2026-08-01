---
title: Pools Module
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, pools, category, color]
sources: [pools.js]
---

# Pools Module

The `pools.js` file manages the dynamic CSS stylesheet for pool category colors. It exports a single function. Its structure mirrors `tags.js`.

## Module-Level Variable

- `_stylesheet` — a reference to the `<style>` element injected into the document head. Initially `null`.

## Function: `refreshCategoryColorMap()`

1. Fetches all pool categories via `PoolCategoryList.get()`.
2. If a previous stylesheet exists, removes it from the document head.
3. Creates a new `<style>` element and appends it to `<head>`.
4. For each pool category in the response results:
   - Generates a CSS class name using `misc.makeCssName(category.name, "pool")`. This produces a class like `pool-category_name`.
   - Inserts a CSS rule: `.{className} { color: <category.color> }`.
5. Returns the promise chain.

This function is called during startup in `main.js` (line 132) to apply pool category colors to all pool links in the UI.

## Dependencies

- `./util/misc.js` — provides `makeCssName()`.
- `./models/pool_category_list.js` — fetches the pool categories from the API.

## Exports

```js
{ refreshCategoryColorMap }
```
