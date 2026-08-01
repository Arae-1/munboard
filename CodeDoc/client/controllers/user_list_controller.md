# User List Controller

**File:** `client/js/controllers/user_list_controller.js`
**Class:** `UserListController`

## Purpose

Controls the users listing page. Shows a paginated, searchable list of registered users.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `users` | `UserListController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Creates a `PageController` instance.
2. Checks if the user has the `user_list` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view users.".
   - Returns early.
4. Activates "users" in the top navigation bar.
5. Sets the page title to "Listing users".
6. Stores the context as `this._ctx`.
7. Creates a `UsersHeaderView` with:
   - `hostNode`: The page controller's header holder node
   - `parameters`: The URL query parameters
8. Registers a `navigate` event listener on the header view.
9. Calls `_syncPageController()`.

---

## Public Methods

### `showSuccess(message)`
Delegates to `this._pageController.showSuccess(message)`.

---

## Event Handlers

### `_evtNavigate(e)`
**Trigger:** User submits the header search form.
**Parameters:** `e.detail.parameters` (search parameters)

**Code path:**
1. Updates the URL without dispatching via `router.showNoDispatch()`.
2. Merges the new parameters into `this._ctx.parameters`.
3. Calls `_syncPageController()`.

---

## Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `this._ctx.parameters` |
| `defaultLimit` | `30` |
| `getClientUrlForPage` | Returns client URL for "users" with offset and limit |
| `requestPage` | Calls `UserList.search(query, offset, limit)` |
| `pageRenderer` | Creates a `UsersPageView` with `canViewUsers: api.hasPrivilege("user_view")` |

### Page Renderer Context

| Property | Source |
|---|---|
| `canViewUsers` | `api.hasPrivilege("user_view")` |

---

## API Calls

| Call | Purpose |
|---|---|
| `UserList.search(query, offset, limit)` | Searches users with the given query, offset, and limit |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `router` | Client-side routing (showNoDispatch) |
| `uri` | Client link formatting |
| `UserList` | Searches users |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `UsersHeaderView` | Renders the search header |
| `UsersPageView` | Renders the user list |
| `EmptyView` | Error view when missing privileges |
