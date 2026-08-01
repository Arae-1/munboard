# Tag Create Controller

**File:** `client/js/controllers/tag_create_controller.js`
**Class:** `TagCreateController`

## Purpose

Controls the tag creation page.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `tag/create` | `TagCreateController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context)

**Code path:**
1. Checks if the user has the `tag_create` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to create tags.".
   - Returns early.
3. Calls `TagCategoryList.get()` to fetch all tag categories.

**Success path:**
- Builds a categories map from the response (key = category name, value = category name).
- Creates a `TagCreateView` with:
  - `canCreate: api.hasPrivilege("tag_create")`
  - `categories`: The map of category names
  - `escapeTagName: uri.escapeTagName`
- Registers a `submit` event listener on the view.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `tag_create` | `canCreate` |

---

## Event Handlers

### `_evtCreate(e)`
**Trigger:** User submits the create form.
**Parameters:** `e.detail` (the form data as a plain object)

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `api.post(uri.formatApiLink("tags"), e.detail)`.

**Success path:**
- Clears the messages again.
- Calls `misc.disableExitConfirmation()`.
- Routes to the tags list page via `router.show(uri.formatClientLink("tags"))`.
- Shows "Tag created." on the list controller.

**Error path:**
- Shows the error message.
- Enables the form.

---

## API Calls

| Call | Endpoint | Purpose |
|---|---|---|
| `TagCategoryList.get()` | (model) | Fetches all tag categories |
| `api.post` | `/api/tags` | Creates a new tag with the form data |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | Privilege checks and API post |
| `misc` | Exit confirmation management |
| `uri` | Link formatting and tag name escaping |
| `TagCategoryList` | Fetches available categories |
| `TagCreateView` | Renders the create form |
| `EmptyView` | Error view when missing privileges |
