---
title: Tags Module
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, tags, category, color]
sources: [tags.js]
---

# Tags Module

The `tags.js` file manages the dynamic CSS stylesheet for tag category colors. It exports a single function.

## Module-Level Variable

- `_stylesheet` — a reference to the `<style>` element injected into the document head. Initially `null`.

## Function: `refreshCategoryColorMap()`

1. Fetches all tag categories via `TagCategoryList.get()`.
2. If a previous stylesheet exists, removes it from the document head.
3. Creates a new `<style>` element and appends it to `<head>`.
4. For each tag category in the response results:
   - Generates a CSS class name using `misc.makeCssName(category.name, "tag")`. This produces a class like `tag-category_name`.
   - Inserts a CSS rule: `.{className} { color: <category.color> }`.
5. Returns the promise chain.

This function is called during startup in `main.js` (line 131) to apply tag category colors to all tag links in the UI.

## Dependencies

- `./util/misc.js` — provides `makeCssName()`.
- `./models/tag_category_list.js` — fetches the tag categories from the API.

## Exports

```js
{ refreshCategoryColorMap }
```
