# CustomFieldInputControl

## UI Element

Manages an inline custom field editor. Replaces a hidden host node with an edit area that contains:
- A name input with autocomplete (`cf-name`)
- A value input for numeric values (`cf-value`)
- An "Add" / "Update" button
- A list (`ul.compact-custom-fields`) of current custom fields

## Constructor

### `constructor(hostNode, customFields, customFieldColors)`
- `hostNode`: The hidden placeholder element.
- `customFields`: An object mapping custom field names to numeric values.
- `customFieldColors`: An object mapping names to CSS color strings.

Calls `CustomFieldAutoCompleteControl.ensureFieldNamesLoaded()` to preload the field name cache.

Renders the `custom-field-input` template. Creates a `CustomFieldAutoCompleteControl` bound to the name input with these options:
- `getTextToFind`: returns the name input value directly.
- `confirm`: sets the name input value to the selected custom field name and stores the selected color.
- `verticalShift`: -2.

Hides the original host node and inserts the edit area after it. Renders existing custom fields from the `customFields` and `customFieldColors` objects, sorted by name.

## Event Bindings (in constructor)

| Event | Element | Action |
|---|---|---|
| `click` | Add/Update button | Calls `_addCurrent()` |
| `keydown` (Enter) | Name input | Calls `_addCurrent()` |
| `keydown` (Enter) | Value input | Calls `_addCurrent()` |
| `input` | Name input | Clears custom validity. Exits edit mode if text changes while editing. |
| `input` | Value input | Clears custom validity |

## Private Methods — Field Operations

### `_addCurrent()`
Validates the name and value inputs:
1. Name must not be empty.
2. Value must be a valid number (NaN shows "Value must be a number").
3. Name must be a valid custom field name (checked via `CustomFieldAutoCompleteControl.isValidFieldName`). Shows "Custom field does not exist" if invalid.

If valid, updates or creates the list item. Stores the selected color. Updates the `customFields` and `customFieldColors` objects. Exits edit mode, focuses the name input, and dispatches a `change` event.

### `_removeField(name)`
Deletes the field name from `customFields` and `customFieldColors`. Removes the corresponding DOM element. If the removed field was the one being edited, exits edit mode. Dispatches a `change` event.

### `_evtAddClick(e)`
Prevents default. Calls `_addCurrent()`.

### `_evtEditClick(name)`
Enters edit mode for an existing field: stores the editing name, populates both inputs, changes button text to "Update", highlights the row, and focuses the value input.

### `_exitEditMode()`
Clears the editing state, resets inputs, restores button text to "Add", and clears row highlighting.

## Private Methods — DOM Manipulation

### `_createListItemNode(name, value, color)`
Creates a `<li>` element with:
- A removal link (`a.remove-custom-field`, pseudo-content `\u00d7`) with a click handler that calls `_removeField`.
- A name span (`span.custom-field-name`) with text `name:` and the color style.
- A value span (`span.custom-field-usages`) with a `data-pseudo-content` attribute set to the value.

Clicking the list item enters edit mode for that field.

### `_updateListItemNode(name, value)`
Finds the `<li>` by `data-name` attribute. Updates the value span's `data-pseudo-content`.

### `_deleteListItemNode(name)`
Finds the `<li>` by `data-name` attribute. Removes it from the DOM.

### `_highlightRow(name)`
Removes the `editing` class from all rows. Adds `editing` to the row with the matching name.

### `_clearHighlight()`
Removes the `editing` class from any row that has it.

## Communication

- **Dispatches**: `change` CustomEvent whenever fields are added or removed.
- **Uses**: `CustomFieldAutoCompleteControl` for autocomplete and field name validation.
- **Uses**: `api.getCustomFieldDefaultColor()` for the default color when none is selected.

## State Management

- `_customFields`: reference to the parent's custom fields object (mutated in place).
- `_customFieldColors`: reference to the parent's color mapping (mutated in place).
- `_selectedColor`: color from the last autocomplete selection.
- `_editingName`: name of the field currently in edit mode. `null` means "add" mode.

## Keyboard Shortcuts

- **Enter** in either input: adds or updates the current field.
