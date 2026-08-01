# Pool List Controller

**File:** `client/js/controllers/pool_list_controller.js`
**Class:** `PoolListController`

## Purpose

Controls the pools listing page. Shows a paginated, searchable list of pools. Supports creating new pools inline from the header.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `pools` | `PoolListController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Creates a `PageController` instance.
2. Checks if the user has the `pool_list` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view pools.".
   - Returns early.
4. Stores the context as `this._ctx`.
5. Activates "pools" in the top navigation bar.
6. Sets the page title to "Listing pools".
7. Creates a `PoolsHeaderView` with:
   - `hostNode`: The page controller's header holder node
   - `parameters`: The URL query parameters
   - `canCreate`: `api.hasPrivilege("pool_create")`
   - `canEditPoolCategories`: `api.hasPrivilege("pool_category_edit")`
8. Registers `submit` and `navigate` event listeners on the header view.
9. Calls `_syncPageController()`.

### Privileges Passed to the Header View

| Privilege | Context Property |
|---|---|
| `pool_create` | `canCreate` |
| `pool_category_edit` | `canEditPoolCategories` |

---

## Public Methods

### `showSuccess(message)`
Delegates to `this._pageController.showSuccess(message)`.

### `showError(message)`
Delegates to `this._pageController.showError(message)`.

---

## Event Handlers

### `_evtSubmit(e)`
**Trigger:** User creates a new pool from the header form.
**Parameters:** `e.detail.pool` (a new Pool instance)

**Code path:**
1. Clears all messages on the view.
2. Disables the form.
3. Calls `e.detail.pool.save()`.

**Success path:**
- Installs the view for editing the new pool.
- Shows "Pool created.".
- Replaces the URL with the new pool's edit page.

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtNavigate(e)`
**Trigger:** User submits the header search/filter form.
**Parameters:** `e.detail.parameters` (search parameters)

**Code path:**
1. Updates the URL without dispatching via `router.showNoDispatch()`.
2. Merges the new parameters into `this._ctx.parameters`.
3. Calls `_syncPageController()` to reload.

---

## Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `this._ctx.parameters` |
| `defaultLimit` | `50` |
| `getClientUrlForPage` | Returns client URL for "pools" with offset and limit |
| `requestPage` | Calls `PoolList.search(query, offset, limit, fields, {})` |
| `pageRenderer` | Creates a `PoolsPageView` |

### Fields Requested

- `id`
- `names`
- `posts`
- `creationTime`
- `postCount`
- `category`

---

## API Calls

| Call | Purpose |
|---|---|
| `PoolList.search(query, offset, limit, fields, {})` | Searches pools with the given query, offset, limit, and fields |
| `pool.save()` | Saves a newly created pool |
| `router.showNoDispatch()` | Updates URL without route dispatch |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing (showNoDispatch, replace) |
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `PoolList` | Searches pools |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `PoolsHeaderView` | Renders the search/create header |
| `PoolsPageView` | Renders the pool list |
| `EmptyView` | Error view when missing privileges |
