# Custom Field Create Controller

**File:** `client/js/controllers/custom_field_create_controller.js`
**Class:** `CustomFieldCreateController`

## Purpose

Controls the custom field creation page.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `custom-field/create` | `CustomFieldCreateController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context)

**Code path:**
1. Checks if the user has the `custom_field_create` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to create custom fields.".
   - Returns early.
3. Creates a `CustomFieldCreateView` with `canCreate: api.hasPrivilege("custom_field_create")`.
4. Registers a `submit` event listener on the view for `_evtCreate`.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `custom_field_create` | `canCreate` |

---

## Event Handlers

### `_evtCreate(e)`
**Trigger:** User submits the create form.
**Parameters:** `e.detail` (the form data as a plain object)

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `api.post(uri.formatApiLink("custom-fields"), e.detail)`.

**Success path:**
- Clears the messages again.
- Calls `misc.disableExitConfirmation()`.
- Routes to the custom fields list page via `router.show(uri.formatClientLink("custom-fields"))`.
- Shows "Custom field created." on the list controller.

**Error path:**
- Shows the error message on the view.
- Enables the form.

---

## API Calls

| Call | Endpoint | Purpose |
|---|---|---|
| `api.post` | `/api/custom-fields` | Creates a new custom field with the form data |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing |
| `api` | Privilege check and API post |
| `misc` | Exit confirmation management |
| `uri` | Link formatting |
| `CustomFieldCreateView` | Renders the create form |
| `EmptyView` | Error view when missing privileges |
