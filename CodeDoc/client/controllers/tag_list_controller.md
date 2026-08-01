# Tag List Controller

**File:** `client/js/controllers/tag_list_controller.js`
**Class:** `TagListController`

## Purpose

Controls the tags listing page. Shows a paginated, searchable list of tags.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `tags` | `TagListController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Creates a `PageController` instance.
2. Checks if the user has the `tag_list` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view tags.".
   - Returns early.
4. Stores the context as `this._ctx`.
5. Activates "tags" in the top navigation bar.
6. Sets the page title to "Listing tags".
7. Creates a `TagsHeaderView` with:
   - `hostNode`: The page controller's header holder node
   - `parameters`: The URL query parameters
   - `canEditTagCategories`: `api.hasPrivilege("tag_category_edit")`
   - `canCreate`: `api.hasPrivilege("tag_create")`
8. Registers a `navigate` event listener on the header view.
9. Calls `_syncPageController()`.

### Privileges Passed to the Header View

| Privilege | Context Property |
|---|---|
| `tag_category_edit` | `canEditTagCategories` |
| `tag_create` | `canCreate` |

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
1. Updates the URL without dispatching via `router.showNoDispatch()`.
2. Merges the new parameters into `this._ctx.parameters`.
3. Calls `_syncPageController()`.

---

## Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `this._ctx.parameters` |
| `defaultLimit` | `50` |
| `getClientUrlForPage` | Returns client URL for "tags" with offset and limit |
| `requestPage` | Calls `TagList.search(query, offset, limit, fields, {})` |
| `pageRenderer` | Creates a `TagsPageView` |

### Fields Requested

- `names`
- `suggestions`
- `implications`
- `creationTime`
- `usages`
- `category`

---

## API Calls

| Call | Purpose |
|---|---|
| `TagList.search(query, offset, limit, fields, {})` | Searches tags with the given query, offset, limit, and fields |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing (showNoDispatch) |
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `TagList` | Searches tags |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `TagsHeaderView` | Renders the search header |
| `TagsPageView` | Renders the tag list |
| `EmptyView` | Error view when missing privileges |
