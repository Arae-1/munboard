# AutoCompleteControl

## UI Element

Manages an autocomplete dropdown suggestion list. The dropdown shows below or above a text input element. It presents suggested completions for text that the user types.

## Constructor

### `constructor(sourceInputNode, options)`

- `sourceInputNode`: The DOM `<input>` element to attach autocomplete to.
- `options`: Configuration object with these keys:
  - `verticalShift` (default `2`): pixel offset between the input and the dropdown.
  - `maxResults` (default `15`): maximum number of suggestions to display.
  - `getTextToFind` (function): returns the text string to match. Default extracts the last word before the cursor position.
  - `confirm` (function): called when the user confirms a suggestion.
  - `delete` (function): called when the user deletes a suggestion via keyboard.
  - `getMatches` (function): returns a `Promise` that resolves to an array of `{caption, value}` objects.

Sets `data-autocomplete` and `autocomplete="off"` attributes on the input. Creates the suggestion `<div>` container and appends it to `document.body`.

## Public Methods

### `hide()`
Removes the suggestion dropdown from display. Sets `_isVisible` to false.

### `replaceSelectedText(result, addSpace)`
Replaces the current word or comma-separated segment in the input with the `result` value. If `addSpace` is false, trims spaces around the result. Positions the cursor at the end of the inserted text.

## Private Methods — Lifecycle

### `_install()`
Throws an error if the input element is null or already has the `data-autocomplete` attribute. Registers `keydown` and `blur` listeners. Creates the suggestion `<div>` with a `<ul>` inside it. Monitors the input node for DOM removal to trigger `_uninstall`.

### `_uninstall()`
Cancels any pending show timeout. Removes the suggestion `<div>` from the DOM.

## Private Methods — Display

### `_show()`
Sets the suggestion `<div>` to `display: block`.

### `_showOrHide()`
Gets the text to find from `options.getTextToFind`. If the text is empty, calls `hide()`. Otherwise, calls `_updateResults`.

### `_updateResults(textToFind)`
Calls `options.getMatches(textToFind)`. Compares new results to old results by JSON hash. If results changed, resets the active result index. Calls `_refreshList`.

### `_refreshList()`
Clears and rebuilds the suggestion `<li>` elements. Each `<li>` gets a `<a>` element with:
- `innerHTML` set to `resultItem.caption`
- `href=""` and `data-key` attribute with the value
- `mouseenter` handler to highlight the item
- `mousedown` handler to confirm the selection

Positions the dropdown relative to the input. Renders the list offscreen first to measure height. The dropdown appears below the input if it is in the upper half of the viewport, otherwise above it. Removes items from the DOM if the list does not fit vertically.

### `_refreshActiveResult()`
Removes the `active` class from all `<li>` elements. Adds the `active` class to the currently selected item.

## Private Methods — Selection

### `_select(newActiveResult)`
Sets `_activeResult` to the new index. Clamps to valid range. Calls `_refreshActiveResult`.

### `_selectPrevious()`
Moves selection to the previous result, wrapping to the last result.

### `_selectNext()`
Moves selection to the next result, wrapping to the first result.

### `_getActiveSuggestion()`
Returns the `value` property of the currently active result, or `null`.

### `_confirm(result)`
Calls `options.confirm(result)` if provided, otherwise calls `defaultConfirmStrategy` (not defined in this class; implemented by subclasses).

### `_delete(result)`
Calls `options.delete(result)` if provided.

## Event Handlers

### `_evtKeyDown(e)`
Handles keyboard input in the source input:

| Key | Action |
|---|---|
| Escape | Hide the dropdown |
| Tab (no shift) | Select next suggestion |
| Shift+Tab | Select previous suggestion |
| ArrowUp | Select previous suggestion |
| ArrowDown | Select next suggestion |
| Enter (with active result) | Confirm the active suggestion, hide |
| Delete (with active result) | Delete the active suggestion, hide |

For any other key, sets a 250ms debounced timer to call `_showOrHide`.

### `_evtBlur(e)`
Clears the show timeout. Hides the dropdown after a 50ms delay.

## Helper Function

### `_getSelectionStart(input)`
Returns the cursor position in the input element. Uses the standard `selectionStart` property. Falls back to Internet Explorer's `document.selection` API.

## Communication

This control does not communicate with controllers or models. It dispatches no events. It acts purely as a DOM utility. It receives `confirm`, `delete`, and `getMatches` callbacks and calls them. Subclasses override `getMatches` to query models.

## State Management

- `_showTimeout`: timer ID for debounced show/hide.
- `_results`: array of current suggestion objects.
- `_activeResult`: index of the highlighted suggestion (`-1` means none).
- `_isVisible`: whether the dropdown is currently displayed.

## Keyboard Shortcuts

See the `_evtKeyDown` table above.
