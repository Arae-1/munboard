# CommentControl

## UI Element

Manages a single comment in the comment system. Renders a comment with editing, preview, deletion, and scoring controls. Uses two templates: `comment` for the main layout and `score` for the vote interface.

## Constructor

### `constructor(hostNode, comment, onlyEditing)`
- `hostNode`: The DOM container for this comment.
- `comment`: A `Comment` model instance. If null, the form shows as a new-comment editor.
- `onlyEditing`: If true, the comment shows in edit mode with no read-only view.

Listens to `change` and `changeScore` events on the comment model.

Renders the template with privilege checks:
- `comment_edit_own` / `comment_edit_any`
- `comment_delete_own` / `comment_delete_any`
- `user_view`

Installs score buttons. If `onlyEditing`, enters edit mode. Otherwise, enters read-only preview mode.

## Public Methods

### `enterEditMode()`
Switches the navigation to "edit" and shows the edit tab.

### `exitEditMode()`
If `onlyEditing`, clears the edit text and stays in edit mode. Otherwise, switches to the read-only "preview" tab and restores the original comment text. Clears validation messages. Resets the height keeper.

### `enableForm()`
Enables all form controls in the comment form.

### `disableForm()`
Disables all form controls in the comment form.

### `showError(message)`
Displays an error message in the host node.

## DOM Getters (all querySelector-based)

| Getter | CSS selector | Purpose |
|---|---|---|
| `_formNode` | `form` | The comment form element |
| `_scoreContainerNode` | `.score-container` | Container for vote buttons |
| `_editButtonNodes` | `li.edit>a, a.edit` | Edit button(s) |
| `_previewEditingButtonNode` | `li.preview>a` | Preview tab button |
| `_deleteButtonNode` | `.delete` | Delete button |
| `_upvoteButtonNode` | `.upvote` | Upvote button |
| `_downvoteButtonNode` | `.downvote` | Downvote button |
| `_saveChangesButtonNode` | `.save-changes` | Save button |
| `_cancelEditingButtonNode` | `.cancel-editing` | Cancel edit button |
| `_textareaNode` | `.tab.edit textarea` | Textarea for markdown input |
| `_contentNode` | `.tab.preview .comment-content` | Rendered content |
| `_heightKeeperNode` | `.keep-height` | Height-preserving wrapper |

## Private Methods

### `_installScore()`
Renders the `score` template inside `_scoreContainerNode`. Binds click handlers on upvote and downvote buttons.

### `_setText(text)`
Sets the textarea value and renders the markdown preview.

### `_selectNav(modeName)`
Toggles the `active` class on all `<nav>` elements that contain the class `modeName`.

### `_selectTab(tabName)`
Toggles the `active` class on all `.tab` and `.tabs li` elements that contain the class `tabName`. Calls `_ensureHeight` to prevent layout jumps.

### `_ensureHeight()`
Locks the `minHeight` of the height-keeper node to prevent layout shift during tab switching.

### `_forgetHeight()`
Resets `minHeight` to null.

## Event Handlers

### `_evtEditClick(e)`
Prevents default navigation. Calls `enterEditMode`.

### `_evtScoreClick(e, score)`
Prevents default. Checks `comment_score` privilege. Dispatches a `score` CustomEvent with the comment and the target score. If the user already voted the same score, the vote is set to 0 (toggle).

### `_evtDeleteClick(e)`
Shows a confirmation dialog. If confirmed, dispatches a `delete` CustomEvent with the comment.

### `_evtChange(e)`
Triggers `exitEditMode` to refresh the display after the comment model changed.

### `_evtChangeScore(e)`
Reinstalls the score display to refresh votes.

### `_evtPreviewEditingClick(e)`
Renders the current textarea content as markdown in the preview pane. Switches to the preview tab.

### `_evtSaveChangesClick(e)`
Dispatches a `submit` CustomEvent with the comment, the control target, and the textarea text.

### `_evtCancelEditingClick(e)`
Calls `exitEditMode`.

## Communication

- **Listens to**: `change` and `changeScore` events from the Comment model.
- **Dispatches**: `score`, `delete`, and `submit` CustomEvents. These are proxied by `CommentListControl` to controllers.

## State Management

The Comment model holds all state. This control does not maintain its own reactive state beyond DOM manipulation.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
