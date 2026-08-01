# Custom Field List Controller

**File:** `client/js/controllers/custom_field_list_controller.js`
**Class:** `CustomFieldListController`

## Purpose

Controls the custom fields listing page. Shows all custom fields with pagination. Supports client-side filtering and sorting.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `custom-fields` | `CustomFieldListController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Creates a `PageController` instance.
2. Checks if the user has the `custom_field_list` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view custom fields.".
   - Returns early.
4. Stores the context as `this._ctx`.
5. Initializes `this._allCustomFields` to `null`.
6. Activates "custom-fields" in the top navigation bar.
7. Sets the page title to "Listing custom fields".
8. Creates a `CustomFieldsHeaderView` with query parameters and `canCreate` privilege.
9. Registers a `navigate` event listener on the header view.
10. Calls `_syncPageController()`.

### Privileges Passed to the Header View

| Privilege | Context Property |
|---|---|
| `custom_field_create` | `canCreate` |

---

## Public Methods

### `showSuccess(message)`
Delegates to `this._pageController.showSuccess(message)`.

### `showError(message)`
Delegates to `this._pageController.showError(message)`.

---

## Event Handlers

### `_evtNavigate(e)`
**Trigger:** User submits the header search/filter form.
**Parameters:** `e.detail.parameters` (search parameters)

**Code path:**
1. Updates the URL without dispatching a route via `router.showNoDispatch()`.
2. Merges the new parameters into `this._ctx.parameters`.
3. Calls `_syncPageController()` to reload the page with new parameters.

---

## Client-Side Filtering and Sorting

### `_filterSort(results, query)`
**Parameters:** `results` (array of custom fields), `query` (the search query string)
**Returns:** Filtered and sorted array of custom fields

This method runs entirely on the client side. It does **not** make any API calls.

**Token parsing:**
1. Trims the query string.
2. Splits the query into tokens by whitespace.
3. Removes tokens that start with `sort:` or `-sort:` (those tokens control sort order).

**Filter tokens:**
- `name:<value>` — filters by name (partial match, case-insensitive)
- `color:<value>` — filters by color (partial match, case-insensitive)
- `-<token>` — negates the filter (excludes matching items)
- Default token (no prefix) — searches name, description, and color (partial match, case-insensitive)

**Sort tokens:**
- `sort:name` — sorts by name ascending (default)
- `-sort:name` — sorts by name descending
- `sort:creation-time` — sorts by creation time ascending
- `-sort:creation-time` — sorts by creation time descending
- `sort:description` — sorts by description ascending
- `-sort:description` — sorts by description descending
- No sort token — sorts by name ascending

---

## Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `this._ctx.parameters` |
| `defaultLimit` | `50` |
| `getClientUrlForPage` | Returns client URL for "custom-fields" with offset and limit |
| `requestPage` | Fetches all custom fields once via `CustomFieldList.get(fields)`, caches them in `this._allCustomFields`, then applies client-side filter/sort, returns all filtered results |
| `pageRenderer` | Creates a `CustomFieldsPageView` |

### Fields Requested

- `id`
- `name`
- `description`
- `color`
- `creationTime`

### Caching Behavior

On the first call to `requestPage`, the controller fetches **all** custom fields from the server and caches them. Subsequent calls use the cached data. Filtering and sorting happen on the client side against the cached data.

---

## API Calls

| Call | Purpose |
|---|---|
| `CustomFieldList.get(fields)` | Fetches all custom fields with specified fields. Called once, then cached. |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing (showNoDispatch) |
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `CustomFieldList` | Fetches all custom fields |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `CustomFieldsHeaderView` | Renders the header with search form |
| `CustomFieldsPageView` | Renders the list of custom fields |
| `EmptyView` | Error view when missing privileges |
