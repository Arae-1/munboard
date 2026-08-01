# Pool Create Controller

**File:** `client/js/controllers/pool_create_controller.js`
**Class:** `PoolCreateController`

## Purpose

Controls the pool creation page.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `pool/create` | `PoolCreateController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context)

**Code path:**
1. Checks if the user has the `pool_create` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to create pools.".
   - Returns early.
3. Calls `PoolCategoryList.get()` to fetch all pool categories.

**Success path:**
- Builds a categories map from the response (key = category name, value = category name).
- Creates a `PoolCreateView` with:
  - `canCreate: api.hasPrivilege("pool_create")`
  - `categories`: The map of category names
  - `escapeTagName: uri.escapeTagName`
- Registers a `submit` event listener on the view.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `pool_create` | `canCreate` |

---

## Event Handlers

### `_evtCreate(e)`
**Trigger:** User submits the create form.
**Parameters:** `e.detail` (the form data as a plain object)

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `api.post(uri.formatApiLink("pool"), e.detail)`.

**Success path:**
- Clears the messages again.
- Calls `misc.disableExitConfirmation()`.
- Routes to the pools list page via `router.show(uri.formatClientLink("pools"))`.
- Shows "Pool created." on the list controller.

**Error path:**
- Shows the error message.
- Enables the form.

---

## API Calls

| Call | Endpoint | Purpose |
|---|---|---|
| `PoolCategoryList.get()` | (model) | Fetches all pool categories |
| `api.post` | `/api/pool` | Creates a new pool with the form data |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | Privilege checks and API post |
| `misc` | Exit confirmation management |
| `uri` | Link formatting and tag name escaping |
| `PoolCategoryList` | Fetches available categories |
| `PoolCreateView` | Renders the create form |
| `EmptyView` | Error view when missing privileges |
