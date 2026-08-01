# Custom Field Controller

**File:** `client/js/controllers/custom_field_controller.js`
**Class:** `CustomFieldController`

## Purpose

Controls the detail page for a single custom field. Supports viewing, editing, and deleting a custom field.

## Routing

| Route | Controller | Section |
|---|---|---|
| `custom-field/:id/edit` | `CustomFieldController` | `"edit"` |
| `custom-field/:id/delete` | `CustomFieldController` | `"delete"` |
| `custom-field/:id` | `CustomFieldController` | `"summary"` |

---

## Constructor

**Parameters:** `ctx` (the router context), `section` (one of `"edit"`, `"delete"`, `"summary"`)

**Code path:**
1. Checks if the user has the `custom_field_view` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view custom fields.".
   - Returns early.
3. Fetches the custom field by ID via `CustomField.get(ctx.parameters.id)`.

**Success path:**
- Activates "custom-fields" in the top navigation bar.
- Sets the page title to "Custom Field: {name}".
- Stores the original name in `this._name`.
- Registers a `change` event listener on the custom field model. This listener calls `_evtSaved`.
- Creates a `CustomFieldView` with the custom field, section, and privileges.
- Registers three event listeners on the view: `change`, `submit`, `delete`.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `custom_field_edit_name` | `canEditAnything` |
| `custom_field_edit_name` | `canEditName` |
| `custom_field_edit_name` | `canEditDescription` |
| `custom_field_edit_name` | `canEditColor` |
| `custom_field_delete` | `canDelete` |
| (config) | `customFieldDefaultColor` |

---

## Event Handlers

### `_evtChange(e)`
**Trigger:** Any change to the view form.
**Code path:** Calls `misc.enableExitConfirmation()` to warn the user before leaving the page with unsaved changes.

### `_evtSaved(e, section)`
**Trigger:** The custom field model fires a `change` event after a successful save.
**Parameters:** `e.detail.customField`

**Code path:**
1. Calls `misc.disableExitConfirmation()`.
2. If the custom field name changed from the original, calls `router.replace()` to update the URL with the new name.

### `_evtUpdate(e)`
**Trigger:** User submits the edit form.
**Parameters:** `e.detail.customField`, `e.detail.name`, `e.detail.description`, `e.detail.color`

**Code path:**
1. Clears all messages on the view.
2. Disables the form.
3. Updates the custom field properties only if they are not `undefined` and not `null`:
   - `name` is set on `e.detail.customField.name`
   - `description` is set on `e.detail.customField.description`
   - `color` is set on `e.detail.customField.color`
4. Calls `e.detail.customField.save()`.

**Success path:** Shows "Custom field saved." and enables the form.
**Error path:** Shows the error message and enables the form.

### `_evtDelete(e)`
**Trigger:** User confirms deletion.
**Parameters:** `e.detail.customField`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.customField.delete()`.

**Success path:** Routes to the custom fields list page and shows "Custom field deleted.".
**Error path:** Shows the error message and enables the form.

---

## API Calls

| Call | Purpose |
|---|---|
| `CustomField.get(id)` | Fetches a single custom field by ID |
| `customField.save()` | Saves edits to the custom field |
| `customField.delete()` | Deletes the custom field |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | Privilege checks and default color |
| `misc` | Exit confirmation management |
| `uri` | Client link formatting |
| `CustomField` | Model for custom field data |
| `top_navigation` | Navigation bar activation and title |
| `CustomFieldView` | Renders the custom field detail page |
| `EmptyView` | Error view when missing privileges |
