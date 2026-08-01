# ExpanderControl

## UI Element

Manages a collapsible section in the sidebar. Groups multiple DOM nodes under a toggleable header. The expand/collapse state persists in `localStorage`.

## Constructor

### `constructor(name, title, nodes)`
- `name`: A unique identifier string. Used as a key in localStorage.
- `title`: The header text for the expander.
- `nodes`: A `NodeList` or array of DOM elements to wrap inside the expander. Filters out null nodes.

If `nodes` is empty after filtering, the constructor returns early and does not create an expander.

Otherwise:
1. Renders the `expander` template with the title.
2. Finds the toggle link, icon element, and content container.
3. Inserts the expander node before the first node in the list.
4. Moves all nodes into the expander content container.
5. Restores the collapsed/expanded state from `localStorage`.
6. Calls `_syncIcon()` to set the correct chevron icon.

## Public Properties

### `title` (setter)
Updates the text of the `<span>` inside the header. Does nothing if the expander was not created.

## Getters

### `_isOpened`
Returns `true` if the expander does NOT have the `collapsed` class.

### `_allStates`
Parses the `expander` key from `localStorage` as JSON. Returns an object where keys are expander names and values are booleans (`true` = opened).

## Private Methods

### `_save()`
Writes the current `_isOpened` value to `localStorage` under the key `expander`, as a JSON object.

### `_syncIcon()`
Toggles the Font Awesome icon classes:
- Opened: `fa-chevron-down` (chevron points down = content is visible).
- Closed: `fa-chevron-up` (chevron points up = content is hidden).

## Event Handlers

### `_evtToggleClick(e)`
Prevents default link navigation. Toggles the `collapsed` class on the expander node. Saves the state to `localStorage`. Calls `_syncIcon()`.

## Communication

Does not communicate with models or controllers. Operates purely on the DOM and `localStorage`. Dispatches no events.

## State Management

State is stored in `localStorage` under the key `"expander"`. Each expander name maps to a boolean value. State persists across page reloads.

## CSS Classes

- `collapsed`: Applied to the expander wrapper when the content is hidden.
- Icon classes: `fa-chevron-down` (opened), `fa-chevron-up` (closed). These are Font Awesome classes.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
