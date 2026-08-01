---
title: Accent Utility
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, utility, accent, favicon, theme]
sources: [accent.js]
---

# Accent Utility

The `accent.js` file applies a user-selected accent color to the page theme and dynamically updates the favicon.

## Function: `apply(color)`

Performs two actions:

### 1. Set CSS Custom Property

Sets the `--accent-color` CSS custom property on `<html>`:

```js
document.documentElement.style.setProperty("--accent-color", color);
```

### 2. Update Favicon

Generates an inline SVG favicon that uses the accent color. The SVG is a stylized geometric icon (a cog-like shape). The accent color is injected into:

- The `stop-color` attribute of a linear gradient (`#swatch9`).
- The `fill` attribute of a complex SVG path (the accent-colored element in the icon).

The SVG is serialized and set as the `href` attribute of `<link rel="shortcut icon">` as a data URI:

```js
link.href = "data:image/svg+xml," + encodeURIComponent(svg);
```

If no favicon `<link>` element exists, the SVG part is generated but not applied.

## Exports

```js
{ apply }
```
