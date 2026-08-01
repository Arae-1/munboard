# PoolInputControl

## UI Element

Manages an inline pool editor. Replaces a hidden host node with an edit area that contains:
- A pool input with autocomplete
- A list (`ul.compact-pools`) of current pools

Each pool list item shows a removal link, a pool page link, a search link with the pool name, and a post count indicator.

## Source Constants

Four string constants identify the origin of a pool addition:

| Constant | Value | Meaning |
|---|---|---|
| `SOURCE_INIT` | `"init"` | Pool was loaded from server data |
| `SOURCE_IMPLICATION` | `"implication"` | Pool was added as a consequence of another action |
| `SOURCE_USER_INPUT` | `"user-input"` | User typed or selected the pool |
| `SOURCE_CLIPBOARD` | `"clipboard"` | Pool was pasted from clipboard |

## Helper Function

### `_fadeOutListItemNodeStatus(listItemNode)`
Removes all CSS classes from the list item after 2500ms. Uses a timeout stored on the node as `fadeTimeout`. If a previous timeout is active, clears it and restarts.

## Constructor

### `constructor(hostNode, poolList)`
- `hostNode`: The hidden placeholder element.
- `poolList`: A `PoolList` model instance.

Renders the `pool-input` template. Creates a `PoolAutoCompleteControl` bound to the pool input with:
- `getTextToFind`: returns the input value directly.
- `confirm`: clears the input and calls `addPool(pool, SOURCE_USER_INPUT)`.
- `delete`: clears the input and calls `deletePool(pool)`.
- `verticalShift`: -2.

Hides the original host node and inserts the edit area after it. Iterates the existing pool list and creates list items for each pool.

## Public Methods

### `addPool(pool, source)`
If the source is not `SOURCE_INIT` and the pool is already in the list, returns immediately. Otherwise:
1. Adds the pool to the `PoolList` model with `this.pools.add(pool, false)`.
2. Creates a list item. If the pool has no category, adds the `new` class.
3. Prepends the item to the pool list.
4. Starts the fadeout timer.
5. Dispatches `add` (with pool and source) and `change` events.

Returns a resolved promise.

### `deletePool(pool)`
If the pool is not in the model, returns immediately. Otherwise:
1. Removes the pool from the model by ID.
2. Hides the autocomplete dropdown.
3. Removes the DOM element.
4. Dispatches `remove` (with pool) and `change` events.

## Private Methods — DOM Manipulation

### `_createListItemNode(pool)`
Creates a `<li>` element containing:
- A removal link (`a.remove-pool`, pseudo-content `\u00d7`) with a click handler that calls `deletePool`.
- A pool page link with a Font Awesome `fa-list-ul` icon and `href` from `uri.formatClientLink("pool", pool.id)`.
- A search link showing the first pool name plus a space. The `href` uses `uri.formatClientLink("posts", {query: "pool:" + pool.searchableName})`.
- A post count span (`span.pool-usages`) with `data-pseudo-content` showing the count.
- Both links get a CSS class from `misc.makeCssName(pool.category, "pool")` if a category exists.

Stores the list item node in `_poolToListItemNode` mapped by each of the pool's names.

### `_deleteListItemNode(pool)`
Finds the list item by pool name. Removes it from the DOM. Deletes all name entries from the `Map`.

### `_getListItemNode(pool)`
Returns the list item node from the `Map` using the pool's first name.

### `_hideAutoComplete()`
Hides the `PoolAutoCompleteControl` dropdown.

## Communication

- **Dispatches**: `add` (detail: `{pool, source}`), `remove` (detail: `{pool}`), and `change` CustomEvents.
- **Uses**: `PoolAutoCompleteControl` for autocomplete.
- **Mutates**: the `PoolList` model directly via `add` and `removeById`.

## State Management

- `_poolToListItemNode`: a `Map` from pool name strings to DOM `<li>` elements.
- `pools`: reference to the parent's `PoolList` model (mutated in place).

## Keyboard Shortcuts

The `PoolAutoCompleteControl` handles autocomplete keyboard shortcuts. No additional shortcuts are defined.
