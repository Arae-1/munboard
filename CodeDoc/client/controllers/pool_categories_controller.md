# Pool Categories Controller

**File:** `client/js/controllers/pool_categories_controller.js`
**Class:** `PoolCategoriesController`

## Purpose

Controls the pool categories management page. Shows all pool categories in an editable list. Supports editing names, colors, adding new categories, and deleting categories. This is the category configuration page, not the pool listing page.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `pool-categories` | `PoolCategoriesController` | Constructor |

---

## Constructor

**Code path:**
1. Checks if the user has the `pool_category_list` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view pool categories.".
   - Returns early.
3. Activates "pools" in the top navigation bar.
4. Sets the page title to "Listing pools".
5. Calls `PoolCategoryList.get()` to fetch all pool categories.

**Success path:**
- Stores the categories as `this._poolCategories` from `response.results`.
- Creates a `PoolCategoriesView` with:
  - `poolCategories`: The fetched categories
  - Privilege flags for edit, delete, create, and set-default operations
- Registers a `submit` event listener on the view.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `pool_category_edit_name` | `canEditName` |
| `pool_category_edit_color` | `canEditColor` |
| `pool_category_delete` | `canDelete` |
| `pool_category_create` | `canCreate` |
| `pool_category_set_default` | `canSetDefault` |

---

## Event Handlers

### `_evtSubmit(e)`
**Trigger:** User submits the category list edits.
**Parameters:** The view fires with the modified pool categories.

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `this._poolCategories.save()` to persist all category changes.

**Success path:**
- Calls `pools.refreshCategoryColorMap()` to update the color map.
- Enables the form.
- Shows "Changes saved.".

**Error path:**
- Enables the form.
- Shows the error message.

---

## API Calls

| Call | Purpose |
|---|---|
| `PoolCategoryList.get()` | Fetches all pool categories |
| `poolCategories.save()` | Saves all changes to pool categories (batch save) |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `pools` | Refreshes the pool category color map after changes |
| `PoolCategoryList` | Fetches and saves pool categories |
| `top_navigation` | Navigation bar activation and title |
| `PoolCategoriesView` | Renders the editable category list |
| `EmptyView` | Error view when missing privileges |
