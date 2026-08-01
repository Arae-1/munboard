# TagInputControl

## UI Element

Manages an inline tag editor. This is the most complex input control. Replaces a hidden host node with an edit area containing:
- A tag input with autocomplete
- A button to add the current text as a tag
- A toggle for suggestion popup opacity (translucent on/off)
- A close button for the suggestion popup
- A list (`ul.compact-tags`) of current tags
- A tag suggestion popup (`ul` inside `.tag-suggestions`) showing related tags with relevance weights

Each tag list item shows a removal link, a tag page link, a search link with the tag name, and a post count indicator.

## Source Constants

Five string constants identify the origin of a tag addition:

| Constant | Value | Meaning |
|---|---|---|
| `SOURCE_INIT` | `"init"` | Tag loaded from server data |
| `SOURCE_IMPLICATION` | `"implication"` | Tag added as a consequence of another tag |
| `SOURCE_USER_INPUT` | `"user-input"` | User typed or selected the tag |
| `SOURCE_SUGGESTION` | `"suggestions"` | Tag added from the suggestion popup |
| `SOURCE_CLIPBOARD` | `"clipboard"` | Tag pasted from clipboard |

## Helper Function

### `_fadeOutListItemNodeStatus(listItemNode)`
Removes all CSS classes from the list item after 2500ms. Uses a timeout stored on the node as `fadeTimeout`. If a previous timeout is active, clears it and restarts.

## SuggestionList Class

### `class SuggestionList`
Internal data structure for managing tag sibling suggestions.

#### `constructor()`
Initializes `_suggestions` (object mapping names to weights) and `_banned` (array of manually dismissed names).

#### `clear()`
Empties all suggestions.

#### `get length()`
Returns the count of current suggestions.

#### `set(suggestion, weight)`
Stores a suggestion with its weight. If it already exists, keeps the maximum weight.

#### `ban(suggestion)`
Adds a suggestion name to the banned list. Banned suggestions do not appear in `getAll`.

#### `getAll()`
Returns an array of `{tagName, weight}` objects sorted by weight descending, then by name ascending. Excludes banned suggestions.

## Constructor

### `constructor(hostNode, tagList)`
- `hostNode`: The hidden placeholder element.
- `tagList`: A `TagList` model instance.

Renders the `tag-input` template. Creates a `TagAutoCompleteControl` bound to the tag input with:
- `getTextToFind`: returns the input value.
- `confirm`: clears input and calls `addTagByName(tag.names[0], SOURCE_USER_INPUT)`.
- `delete`: clears input and calls `deleteTag(tag)`.
- `verticalShift`: -2.
- `isTaggedWith`: checks if the tag list already contains a tag name.
- `isNegationAllowed`: false.

Binds DOM events:
- `keydown` on tag input (Space/Enter to add).
- `paste` on tag input.
- `click` on the opacity toggle link.
- `click` on the close suggestions link.
- `click` on the "Add tag" button.

Hides the host node and inserts the edit area after it. Iterates the existing tag list and creates list items.

## Public Methods

### `addTagByText(text, source)`
Splits the text by whitespace. Adds each word as a tag name (in reverse order). Calls `addTagByName` for each.

### `addTagByName(name, source)`
Trims the name. Rejects empty names and the names `.` and `..`.

Calls `Tag.get(name)`. On success, calls `addTag` with the full tag object. On failure, creates a new `Tag` with only the name set (no category) and calls `addTag` with it.

### `addTag(tag, source)`
If the source is not `SOURCE_INIT` and the tag is already in the list:
- Highlights the existing list item as `duplicate` (unless `SOURCE_IMPLICATION`).
- Returns a resolved promise.

Otherwise:
1. Calls `this.tags.addByName(tag.names[0], false)` to add to the model.
2. Creates a list item with a CSS class based on the source: `new` (no category), `implication` (from implication), or `added` (user added).
3. Prepends the item to the tag list.
4. Starts the fadeout timer.
5. Resolves all tag implications via `Promise.all`, recursively adding each implication tag with `SOURCE_IMPLICATION`.
6. Dispatches `add` (with tag and source) and `change` events.

### `deleteTag(tag)`
If the tag is not in the model, returns. Otherwise:
1. Removes the tag from the model by name.
2. Hides the autocomplete dropdown.
3. Removes the DOM element.
4. Dispatches `remove` (with tag) and `change` events.

## Private Methods — Tag List DOM

### `_createListItemNode(tag)`
Creates a `<li>` element with:
- A removal link (`a.remove-tag`, pseudo-content `\u00d7`).
- A tag page link with Font Awesome `fa-tag` icon.
- A search link showing the tag name. Its click handler clears suggestions and loads new suggestions for this tag.

  If `tag.postCount > 0`, loads sibling suggestions and removes the popup opacity. Otherwise, closes the suggestion popup.

- A post count span (`span.tag-usages`) with `data-pseudo-content` set to the count.
- Both links get a CSS class from `misc.makeCssName(tag.category, "tag")`.
- Stores the list item in `_tagToListItemNode` mapped by each tag name.

### `_deleteListItemNode(tag)`
Finds the list item by name. Removes it from the DOM. Deletes all name entries from the `Map`.

### `_getListItemNode(tag)`
Returns the list item from the `Map` using the tag's first name.

## Private Methods — Suggestions

### `_loadSuggestions(tag)`
If tag suggestions are disabled in settings, returns.

Calls `api.get(uri.formatApiLink("tag-siblings", tag.names[0]))` with `noProgress: true`. On success, processes sibling data:
1. Computes `maxSiblingOccurrences` as the maximum occurrence count among siblings.
2. Adds each sibling to the suggestion list with weight: `(occurrences * 4.9) / maxSiblingOccurrences`.
3. Adds explicit tag suggestions with weight 5.

Opens the suggestion popup if there are suggestions. Otherwise, closes it.

### `_refreshSuggestionsPopup()`
Rebuilds the suggestion popup content:
- Clears and resets the scroll position.
- For each suggestion, excludes already-tagged tags.
- Creates list items with:
  - A removal link (bans the suggestion).
  - A weight span with `data-pseudo-content`.
  - An "add tag" link that adds the tag with `SOURCE_SUGGESTION` source.
- Tag category colors are resolved asynchronously via `Tag.get(tagName)`.

### `_closeSuggestionsPopup()`
Clears suggestions and removes the `shown` class.

### `_removeSuggestionsPopupOpacity()`
Removes the `translucent` class.

### `_toggleSuggestionsPopupOpacity()`
Toggles the `translucent` class.

### `_openSuggestionsPopup()`
Adds the `shown` class and calls `_refreshSuggestionsPopup`.

## Private Methods — Other

### `_hideAutoComplete()`
Hides the `TagAutoCompleteControl` dropdown.

## Event Handlers

### `_evtInputPaste(e)`
Prevents default. Extracts plain text from clipboard (handles both `window.clipboardData` for IE and `e.clipboardData`). If pasted text exceeds 20,000 characters, shows an alert. Otherwise, hides autocomplete and calls `addTagByText` with `SOURCE_CLIPBOARD`. Clears the input.

### `_evtCloseSuggestionsPopupClick(e)`
Closes the suggestion popup.

### `_evtAddTagButtonClick(e)`
Adds the current input value as a tag with `SOURCE_USER_INPUT`. Clears the input.

### `_evtToggleSuggestionsPopupOpacityClick(e)`
Toggles the suggestion popup opacity.

### `_evtInputKeyDown(e)`
If Enter or Space is pressed:
1. Prevents default.
2. Hides autocomplete.
3. Calls `addTagByText` with `SOURCE_USER_INPUT`.
4. Clears the input.

## Communication

- **Dispatches**: `add` (detail: `{tag, source}`), `remove` (detail: `{tag}`), and `change` CustomEvents.
- **Uses**: `TagAutoCompleteControl` for autocomplete.
- **Uses**: `Tag.get(name)` to fetch full tag data before adding.
- **Uses**: `api.get(...)` to load tag sibling suggestions.
- **Uses**: `settings.get()` to check if tag suggestions are enabled.
- **Mutates**: the `TagList` model via `addByName` and `removeByName`.

## State Management

- `_tagToListItemNode`: a `Map` from tag name strings to DOM `<li>` elements.
- `_suggestions`: a `SuggestionList` instance tracking current tag sibling suggestions and banned suggestions.
- `tags`: reference to the parent's `TagList` model (mutated in place).

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Enter | Adds input text as a tag |
| Space | Adds input text as a tag |
| Tag link click | Opens/loads sibling suggestions |
