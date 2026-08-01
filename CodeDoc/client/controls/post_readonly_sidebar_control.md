# PostReadonlySidebarControl

## UI Element

Manages the read-only sidebar for a post view page. Displays:
- Post score with upvote/downvote buttons
- Favorite count with add/remove buttons
- Safety rating indicator
- Tag list with category colors and links
- Custom fields with colors
- Pool list with post counts
- Relations
- Source links
- Post dates
- Fit mode buttons (zoom: fit both, original, fit width, fit height)

Uses templates: `post-readonly-sidebar`, `score`, and `fav`.

## Constructor

### `constructor(hostNode, post, postContentControl)`
- `hostNode`: The sidebar DOM container.
- `post`: The Post model.
- `postContentControl`: A `PostContentControl` instance for fit mode control.

Listens to `changeFavorite` and `changeScore` on the post model.

Renders the template with privilege checks:
- `post_list`, `post_edit`, `tag_view`, `custom_field_list`
- `post_score` (enables voting)
- `post_favorite` (enables favoriting)
- `api.safetyEnabled()` (controls safety display)

Installs favorite system, score system, and fit mode buttons. Calls `_syncFitButton` to highlight the active zoom mode.

## DOM Getters

| Getter | Selector |
|---|---|
| `_scoreContainerNode` | `.score-container` |
| `_favContainerNode` | `.fav-container` |
| `_upvoteButtonNode` | `.upvote` |
| `_downvoteButtonNode` | `.downvote` |
| `_addFavButtonNode` | `.add-favorite` |
| `_remFavButtonNode` | `.remove-favorite` |
| `_fitBothButtonNode` | `.fit-both` |
| `_fitOriginalButtonNode` | `.fit-original` |
| `_fitWidthButtonNode` | `.fit-width` |
| `_fitHeightButtonNode` | `.fit-height` |

## Private Methods

### `_installFav()`
Renders the `fav` template with `favoriteCount`, `ownFavorite`, and `canFavorite`. Binds click handlers on the add and remove favorite buttons.

### `_installScore()`
Renders the `score` template with `score`, `ownScore`, and `canScore`. Binds click handlers on upvote and downvote buttons.

### `_installFitButtons()`
Binds click handlers on all four fit mode buttons. Each handler calls `_eventZoomProxy` with the corresponding `PostContentControl` fit method.

### `_eventZoomProxy(func)`
Returns a click event handler that:
1. Prevents default.
2. Blurs the button (removes focus ring).
3. Calls the fit function.
4. Calls `_syncFitButton`.
5. Dispatches a `fitModeChange` event with the mode name.

### `_getFitMode()`
Maps the active fit function reference back to a mode name string: `"fit-both"`, `"fit-original"`, `"fit-width"`, or `"fit-height"`.

### `_syncFitButton()`
Removes the `active` class from the current zoom link. Adds `active` to the link matching the current fit mode.

## Event Handlers

### `_evtAddToFavoritesClick(e)`
Dispatches a `favorite` CustomEvent with the post.

### `_evtRemoveFromFavoritesClick(e)`
Dispatches an `unfavorite` CustomEvent with the post.

### `_evtScoreClick(e, score)`
Dispatches a `score` CustomEvent with the post and the target score. Toggles: if the user already voted the same score, the vote is set to 0.

### `_evtChangeFav(e)`
Reinstalls the favorite display to reflect updated counts.

### `_evtChangeScore(e)`
Reinstalls the score display to reflect updated votes.

## Communication

- **Dispatches**: `favorite`, `unfavorite`, `score`, `fitModeChange` CustomEvents.
- **Listens to**: `changeFavorite` and `changeScore` events from the Post model.
- **Controls**: `PostContentControl` via direct method calls for fit modes.

## State Management

No persistent local state. Delegates to the Post model and PostContentControl for all state.

## Keyboard Shortcuts

No built-in keyboard shortcuts.
