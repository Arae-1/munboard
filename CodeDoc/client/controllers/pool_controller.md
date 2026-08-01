# Pool Controller

**File:** `client/js/controllers/pool_controller.js`
**Class:** `PoolController`

## Purpose

Controls the detail page for a single pool. Supports viewing, editing, merging, and deleting a pool.

## Routing

| Route | Controller | Section |
|---|---|---|
| `pool/:id/edit` | `PoolController` | `"edit"` |
| `pool/:id/merge` | `PoolController` | `"merge"` |
| `pool/:id/delete` | `PoolController` | `"delete"` |
| `pool/:id` | `PoolController` | `"summary"` |

---

## Constructor

**Parameters:** `ctx` (the router context), `section` (one of `"edit"`, `"merge"`, `"delete"`, `"summary"`)

**Code path:**
1. Checks if the user has the `pool_view` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view pools.".
   - Returns early.
3. Calls `Promise.all([PoolCategoryList.get(), Pool.get(ctx.parameters.id)])` to fetch both pool categories and the pool data in parallel.

**Success path:**
- Activates "pools" in the top navigation bar.
- Sets the page title to "Pool #{first_name}".
- Stores the original first name in `this._name` (using `ctx.parameters.name`).
- Registers a `change` event listener on the pool model for `_evtSaved`.
- Builds a categories map from the pool categories response (key = category name, value = category name).
- Creates a `PoolView` with pool data, section, privileges, and categories.
- Registers four event listeners on the view: `change`, `submit`, `merge`, `delete`.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `pool_edit` | `canEditAnything` |
| `pool_edit_name` | `canEditNames` |
| `pool_edit_category` | `canEditCategory` |
| `pool_edit_description` | `canEditDescription` |
| `pool_edit_post` | `canEditPosts` |
| `pool_edit_launch_date` | `canEditLaunchDate` |
| `pool_merge` | `canMerge` |
| `pool_delete` | `canDelete` |

---

## Event Handlers

### `_evtChange(e)`
**Trigger:** Any change to the view form.
**Code path:** Calls `misc.enableExitConfirmation()`.

### `_evtSaved(e, section)`
**Trigger:** The pool model fires a `change` event after a successful save.
**Parameters:** `e.detail.pool`

**Code path:**
1. Calls `misc.disableExitConfirmation()`.
2. If the pool's first name changed from the original, calls `router.replace()` to update the URL.

### `_evtUpdate(e)`
**Trigger:** User submits the edit form.
**Parameters:** `e.detail.pool`, `e.detail.names`, `e.detail.category`, `e.detail.description`, `e.detail.posts`, `e.detail.launchDate`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Updates pool properties (only if not `undefined`/`null`):
   - `names` — sets `e.detail.pool.names`
   - `category` — sets `e.detail.pool.category`
   - `description` — sets `e.detail.pool.description`
   - `posts` — clears the current posts list, then adds each post ID as a `Post.fromResponse({ id })`
   - `launchDate` — sets `e.detail.pool.launchDate` (note: `null` is allowed for this field, checked with `!== undefined` only)
4. Calls `e.detail.pool.save()`.

**Success path:** Shows "Pool saved." and enables the form.
**Error path:** Shows the error message and enables the form.

### `_evtMerge(e)`
**Trigger:** User confirms a pool merge.
**Parameters:** `e.detail.pool`, `e.detail.targetPoolId`, `e.detail.addAlias`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.pool.merge(targetPoolId, addAlias)`.

**Success path:**
- Shows "Pool merged.".
- Enables the form.
- Replaces the URL with the target pool's merge page.

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtDelete(e)`
**Trigger:** User confirms pool deletion.
**Parameters:** `e.detail.pool`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.pool.delete()`.

**Success path:** Routes to the pools list page and shows "Pool deleted.".
**Error path:** Shows the error message and enables the form.

---

## API Calls

| Call | Purpose |
|---|---|
| `PoolCategoryList.get()` | Fetches all pool categories |
| `Pool.get(id)` | Fetches a single pool by ID |
| `pool.save()` | Saves edits to the pool |
| `pool.merge(targetPoolId, addAlias)` | Merges the pool into another pool |
| `pool.delete()` | Deletes the pool |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | Privilege checks |
| `misc` | Exit confirmation management |
| `uri` | Client link formatting and tag name escaping |
| `Pool` | Pool model |
| `Post` | Post model (for pool.posts) |
| `PoolCategoryList` | Fetches pool categories |
| `top_navigation` | Navigation bar activation and title |
| `PoolView` | Renders the pool detail page |
| `EmptyView` | Error view when missing privileges |
