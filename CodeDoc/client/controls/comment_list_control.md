# CommentListControl

## UI Element

Manages a list of comments rendered inside a host node. Uses the `comment-list` template for the container structure. Creates individual `CommentControl` instances for each comment in the model list.

## Constructor

### `constructor(hostNode, comments, reversed)`
- `hostNode`: The DOM container for the comment list.
- `comments`: A `CommentList` model instance (iterable, with add/remove tracking).
- `reversed`: If true, renders comments in reverse order (newest first).

Replaces the host node content with the `comment-list` template. Iterates the comment model list and installs a `CommentControl` into a new `<li>` for each comment.

Listens to `add` and `remove` events on the `comments` model.

## DOM Getters

### `_commentListNode`
Returns the `<ul>` element inside the host node.

## Private Methods

### `_installCommentNode(comment)`
Creates a new `<li>` element. Instantiates a `CommentControl` inside it with `onlyEditing` set to false. Proxies `submit`, `score`, and `delete` events from the `CommentControl` to this `CommentListControl`. Stores a map entry from the comment ID to its `<li>` element. Appends the `<li>` to the list.

### `_uninstallCommentNode(comment)`
Removes the `<li>` element for the comment from the DOM using the ID-to-node map.

## Event Handlers

### `_evtAdd(e)`
Calls `_installCommentNode(e.detail.comment)` when a new comment is added to the model.

### `_evtRemove(e)`
Calls `_uninstallCommentNode(e.detail.comment)` when a comment is removed from the model.

## Communication

- **Listens to**: `add` and `remove` events from the `CommentList` model.
- **Proxies events**: The `submit`, `score`, and `delete` events from each `CommentControl` are proxied upward. `events.proxyEvent` forwards these events so the controller that owns the `CommentListControl` can handle them.

## State Management

Maintains `_commentIdToNode`, an object mapping comment IDs to their DOM `<li>` elements. No other persistent state.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
