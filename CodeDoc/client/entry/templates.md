---
title: Templates Wrapper
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, templates]
sources: [templates.js]
---

# Templates Wrapper

The `templates.js` file is a thin wrapper that re-exports the auto-generated templates module.

## Code

```js
module.exports = require("./.templates.autogen.js");
```

## Dependencies

- `./.templates.autogen.js` — an auto-generated file that contains compiled Underscore templates as named functions (e.g., `templates['comment']`, `templates['post-list']`). This file is ~2700 lines long and is produced by the build system.

## Usage in views.js

The `getTemplate()` function in `util/views.js` imports this module to resolve template names and render them with a context object. The context object is enriched with helper functions (`makeRadio`, `makeCheckbox`, `makeRelativeTime`, etc.) before the template factory is called.

## Exports

The compiled templates hash object. Key = template name (string), Value = function that takes a `ctx` object and returns an HTML string.
