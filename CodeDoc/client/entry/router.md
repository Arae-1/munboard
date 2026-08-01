---
title: Client Router
type: entity
created: 2026-08-01
updated: 2026-08-01
tags: [client, entry, router, routing]
sources: [router.js]
---

# Client Router

The `router.js` file defines the client-side routing system. It is a modified fork of the `page.js` library by visionmedia. The file exports a singleton `Router` instance.

## Modifications from page.js

- Changed regexes to path components.
- Removed unused features.
- Refactored to ES6 classes.
- Added the ability to call `.save()` (now `.replaceState()`) in `.exit()` handlers without side effects.
- Page refresh recovers state from history.
- Renamed `.save()` to `.replaceState()`.
- Exposed `.url` property.

## Functions

### `_getOrigin()`

Returns the current page origin as `protocol//hostname:port`.

### `_isSameOrigin(href)`

Returns `true` if the given `href` starts with the current page origin.

### `_getBaseHref()`

Returns the href attribute of the first `<base>` element, minus the origin and trailing slashes. Returns an empty string if no `<base>` element exists.

## Class: Context

A `Context` object holds the state for one navigation.

### Constructor: `Context(path, state)`

- Prepends the base href to the path if needed.
- Sets `canonicalPath` (path with base) and `path` (path without base).
- Sets `title` from `document.title`.
- Sets `state` from the given state or an empty object. The `state.path` field is set to the path.
- Initializes `parameters` as an empty object (populated during route matching).

### Method: `pushState()`

Calls `history.pushState()` with the current state, title, and canonical path.

### Method: `replaceState()`

Calls `history.replaceState()` with the current state, title, and canonical path.

## Class: Route

A `Route` object matches URL paths and carries middleware functions.

### Constructor: `Route(path)`

- Sets `method` to `"GET"`.
- Stores the `path` array (list of path components).
- If `path` is `null`, the regex matches everything (`/.*/`).
- If `path` is an array:
  - Components starting with `:` are treated as named parameters (e.g., `:id`).
  - Other components are literal matches.
  - Generates a regex that also extracts a trailing key=value segment delimited by `;` into a parameter named `variable`.

### Method: `middleware(fn)`

Returns a middleware function `(ctx, next)`. If the route matches `ctx.path`, the function calls `fn(ctx, next)`. Otherwise, it calls `next()`.

### Method: `match(path, parameters)`

- Removes the query string from the path.
- Runs the route regex against the pathname.
- If no match, returns `false`.
- If matched, extracts named parameter values, URL-decodes them, and stores them in the `parameters` object.
- The special `variable` parameter parses semicolon-delimited key=value pairs.

## Class: Router

The main routing class. One singleton instance is exported.

### Constructor: `Router()`

Initializes `_callbacks` (enter middleware list) and `_exits` (exit middleware list).

### Method: `enter(path, ...middleware)`

Creates a `Route` from `path` and pushes all subsequent arguments as enter middleware for that route.

### Method: `exit(path, ...middleware)`

Creates a `Route` from `path` and pushes all subsequent arguments as exit middleware for that route.

### Method: `start()`

- If already running, returns.
- Sets `_running = true`.
- Registers `popstate` and click event listeners.
- Navigates to the current URL via `this.replace()`. Returns the context.

### Method: `stop()`

- If not running, returns.
- Removes the `popstate` and click event listeners.
- Sets `_running = false`.

### Method: `showNoDispatch(path, state)`

Creates a context, pushes the state to history, stores it as `this.ctx`, but does not dispatch middleware. Returns the context.

### Method: `show(path, state, push)`

Creates a context. Calls `dispatch()`. After dispatch, pushes state if `push` is not `false` and the path changed. Returns the context.

### Method: `replace(path, state, dispatch)`

Creates a context. If `dispatch` is truthy, dispatches middleware then calls `ctx.replaceState()`. Otherwise, just calls `ctx.replaceState()`. Returns the context.

### Method: `dispatch(ctx, middle)`

Builds the call chain:
1. Exit callbacks (if there is an existing context).
2. A swap callback that replaces `this.ctx` and calls `middle()`.
3. Enter callbacks.
4. `_unhandled` handler.
5. A no-op final callback.

Executes the chain sequentially via a recursive closure.

### Method: `_unhandled(ctx, next)`

If the current browser location path differs from the context canonical path, stops the router and sets `location.href` for a full page navigation.

### Getter: `url`

Returns `location.pathname + location.search + location.hash`.

## Function: `_onPopState(router)`

Returns a `popstate` event handler. Waits for the document `load` event before handling popstate events. On popstate:
- If `e.state` exists, calls `router.replace()` with the state's path.
- Otherwise, calls `router.show()` with the current pathname and hash.

## Function: `_onClick(router)`

Returns a click event handler that intercepts same-origin link clicks and routes them through `router.show()` instead of a full page load.

Ignores clicks when:
- Button is not the primary button (0).
- Meta, Ctrl, or Shift keys are pressed.
- Event default was prevented.
- The `<a>` element has a `download` attribute or `rel="external"`.
- The link is to the same pathname with a hash.
- The link uses `mailto:`.
- The element has a `target` attribute.
- The link is to a different origin.
- The resolved path matches the base href (meaning it is an external link).

## Exports

The file exports a singleton `Router` instance created with `new Router()`. The `Router` prototype is augmented with:
- `Router.prototype.Context = Context`
- `Router.prototype.Route = Route`
