# Tag Categories Controller

**File:** `client/js/controllers/tag_categories_controller.js`
**Class:** `TagCategoriesController`

## Purpose

Controls the tag categories management page. Shows all tag categories in an editable list. Supports editing names, colors, order, adding new categories, deleting categories, and setting the default category.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `tag-categories` | `TagCategoriesController` | Constructor |

---

## Constructor

**Code path:**
1. Checks if the user has the `tag_category_list` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view tag categories.".
   - Returns early.
3. Activates "tags" in the top navigation bar.
4. Sets the page title to "Listing tags".
5. Calls `TagCategoryList.get()` to fetch all tag categories.

**Success path:**
- Stores the categories as `this._tagCategories` from `response.results`.
- Creates a `TagCategoriesView` with:
  - `tagCategories`: The fetched categories
  - Privilege flags for edit, delete, create, set-order, and set-default operations
- Registers a `submit` event listener on the view.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `tag_category_edit_name` | `canEditName` |
| `tag_category_edit_color` | `canEditColor` |
| `tag_category_edit_order` | `canEditOrder` |
| `tag_category_delete` | `canDelete` |
| `tag_category_create` | `canCreate` |
| `tag_category_set_default` | `canSetDefault` |

---

## Event Handlers

### `_evtSubmit(e)`
**Trigger:** User submits the category list edits.
**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `this._tagCategories.save()` to persist all changes.

**Success path:**
- Calls `tags.refreshCategoryColorMap()` to update the color map.
- Enables the form.
- Shows "Changes saved.".

**Error path:**
- Enables the form.
- Shows the error message.

---

## API Calls

| Call | Purpose |
|---|---|
| `TagCategoryList.get()` | Fetches all tag categories |
| `tagCategories.save()` | Saves all changes to tag categories (batch save) |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `tags` | Refreshes the tag category color map after changes |
| `TagCategoryList` | Fetches and saves tag categories |
| `top_navigation` | Navigation bar activation and title |
| `TagCategoriesView` | Renders the editable category list |
| `EmptyView` | Error view when missing privileges |
