# Tag Controller

**File:** `client/js/controllers/tag_controller.js`
**Class:** `TagController`

## Purpose

Controls the detail page for a single tag. Supports viewing, editing, merging, and deleting a tag.

## Routing

| Route | Controller | Section |
|---|---|---|
| `tag/:name/edit` | `TagController` | `"edit"` |
| `tag/:name/merge` | `TagController` | `"merge"` |
| `tag/:name/delete` | `TagController` | `"delete"` |
| `tag/:name` | `TagController` | `"summary"` |

---

## Constructor

**Parameters:** `ctx` (the router context), `section` (one of `"edit"`, `"merge"`, `"delete"`, `"summary"`)

**Code path:**
1. Checks if the user has the `tag_view` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view tags.".
   - Returns early.
3. Calls `Promise.all([TagCategoryList.get(), Tag.get(ctx.parameters.name)])` to fetch both tag categories and the tag data in parallel.

**Success path:**
- Activates "tags" in the top navigation bar.
- Sets the page title to "Tag #{first_name}".
- Stores the original name in `this._name` (using `ctx.parameters.name`).
- Registers a `change` event listener on the tag model for `_evtSaved`.
- Builds a categories map from the tag categories response (key = category name, value = category name).
- Creates a `TagView` with tag data, section, privileges, and categories.
- Registers four event listeners on the view: `change`, `submit`, `merge`, `delete`.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `tag_edit` | `canEditAnything` |
| `tag_edit_name` | `canEditNames` |
| `tag_edit_category` | `canEditCategory` |
| `tag_edit_implication` | `canEditImplications` |
| `tag_edit_suggestion` | `canEditSuggestions` |
| `tag_edit_description` | `canEditDescription` |
| `tag_merge` | `canMerge` |
| `tag_delete` | `canDelete` |

---

## Event Handlers

### `_evtChange(e)`
**Trigger:** Any change to the view form.
**Code path:** Calls `misc.enableExitConfirmation()`.

### `_evtSaved(e, section)`
**Trigger:** The tag model fires a `change` event after a successful save.
**Parameters:** `e.detail.tag`

**Code path:**
1. Calls `misc.disableExitConfirmation()`.
2. If the tag's first name changed from the original, calls `router.replace()` to update the URL with the new name.

### `_evtUpdate(e)`
**Trigger:** User submits the edit form.
**Parameters:** `e.detail.tag`, `e.detail.names`, `e.detail.category`, `e.detail.description`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Updates tag properties (only if not `undefined` and not `null`):
   - `names` — sets `e.detail.tag.names`
   - `category` — sets `e.detail.tag.category`
   - `description` — sets `e.detail.tag.description`
4. Calls `e.detail.tag.save()`.

**Success path:** Shows "Tag saved." and enables the form.
**Error path:** Shows the error message and enables the form.

### `_evtMerge(e)`
**Trigger:** User confirms a tag merge.
**Parameters:** `e.detail.tag`, `e.detail.targetTagName`, `e.detail.addAlias`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.tag.merge(targetTagName, addAlias)`.

**Success path:**
- Shows "Tag merged.".
- Enables the form.
- Replaces the URL with the target tag's merge page.

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtDelete(e)`
**Trigger:** User confirms tag deletion.
**Parameters:** `e.detail.tag`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.tag.delete()`.

**Success path:** Routes to the tags list page and shows "Tag deleted.".
**Error path:** Shows the error message and enables the form.

---

## API Calls

| Call | Purpose |
|---|---|
| `TagCategoryList.get()` | Fetches all tag categories |
| `Tag.get(name)` | Fetches a single tag by name |
| `tag.save()` | Saves edits to the tag |
| `tag.merge(targetTagName, addAlias)` | Merges the tag into another tag |
| `tag.delete()` | Deletes the tag |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | Privilege checks |
| `misc` | Exit confirmation management |
| `uri` | Client link formatting and tag name escaping |
| `Tag` | Tag model |
| `TagCategoryList` | Fetches tag categories |
| `top_navigation` | Navigation bar activation and title |
| `TagView` | Renders the tag detail page |
| `EmptyView` | Error view when missing privileges |
