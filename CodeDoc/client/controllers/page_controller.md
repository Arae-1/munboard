# Page Controller

**File:** `client/js/controllers/page_controller.js`
**Class:** `PageController`

## Purpose

Provides pagination logic for list controllers. Controls whether pagination uses endless scroll or manual page buttons. This is a shared utility controller, not a route controller.

## Routing

This controller does **not** register any routes. It is instantiated by other controllers as a helper.

---

## Constructor

**Code path:**
1. Reads the `endlessScroll` setting from `settings.get()`.
2. If `endlessScroll` is true:
   - Creates an `EndlessPageView` instance.
3. If `endlessScroll` is false:
   - Creates a `ManualPageView` instance.

---

## Properties

### `view` (getter)
**Returns:** `this._view` (either `EndlessPageView` or `ManualPageView`)

This property is used by list controllers to access the page view's DOM nodes (e.g., `pageHeaderHolderNode`).

---

## Public Methods

### `run(ctx)`
**Parameters:** `ctx` (a configuration object)

**Code path:** Delegates to `this._view.run(ctx)`. The view handles all pagination logic.

The configuration object must include:
- `parameters`: URL query parameters
- `defaultLimit`: Default number of items per page
- `getClientUrlForPage`: Function that generates the client URL for a page
- `requestPage`: Function that fetches a page of data
- `pageRenderer`: Function that renders a page of items into a view

### `showSuccess(message)`
**Parameters:** `message` (string)
**Code path:** Delegates to `this._view.showSuccess(message)`.

### `showError(message)`
**Parameters:** `message` (string)
**Code path:** Delegates to `this._view.showError(message)`.

---

## Dependencies

| Module | Use |
|---|---|
| `settings` | Reads the `endlessScroll` setting |
| `EndlessPageView` | View for endless scroll pagination |
| `ManualPageView` | View for manual page button pagination |
