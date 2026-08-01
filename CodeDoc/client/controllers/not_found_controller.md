# Not Found Controller

**File:** `client/js/controllers/not_found_controller.js`
**Class:** `NotFoundController`

## Purpose

Controls the 404 Not Found page. Shows when no route matches the current URL.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `null` (fallback) | `NotFoundController` | Constructor |

The route uses `null` as the path pattern. This means the router enters this controller only when no other route matches.

---

## Constructor

**Parameters:** `path` (the canonical path that was not found, from `ctx.canonicalPath`)

**Code path:**
1. Activates an empty string `""` in the top navigation bar (clears the active item).
2. Sets the page title to "Not found".
3. Creates a `NotFoundView` with the path that was not found.

### Notes

- No privilege check. The page is always accessible.
- No event listeners.
- No API calls.
- No error handling. The view is purely informational.

---

## Dependencies

| Module | Use |
|---|---|
| `top_navigation` | Navigation bar activation (empty) and title |
| `NotFoundView` | Renders the 404 page |
