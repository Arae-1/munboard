# Client Controls

Controls are reusable UI components that manage DOM interactions, event handling, and data display. They bridge the gap between views (templates) and the controller/model layer.

There are 15 controls organized into three layers: autocomplete controls, input controls, and display controls.

## Control Architecture

Controls follow a layered design:

```
AutoCompleteControl (base)
  ├── TagAutoCompleteControl
  ├── PoolAutoCompleteControl
  └── CustomFieldAutoCompleteControl

Input Controls (use autocomplete controls)
  ├── TagInputControl ────── uses TagAutoCompleteControl
  ├── PoolInputControl ───── uses PoolAutoCompleteControl
  └── CustomFieldInputControl uses CustomFieldAutoCompleteControl

Display Controls
  ├── CommentControl
  ├── CommentListControl ──── contains CommentControl instances
  ├── PostContentControl
  ├── PostReadonlySidebarControl
  ├── PostEditSidebarControl ─ contains TagInputControl, PoolInputControl,
  │                             CustomFieldInputControl, FileDropperControl
  ├── PostNotesOverlayControl
  ├── FileDropperControl
  └── ExpanderControl
```

## Event Handling

Controls communicate through the DOM `CustomEvent` API. They do not call controllers or models directly. They dispatch events upward and listen to model events for data changes.

### Event Flow

```
Control dispatches CustomEvent
  → parent control receives it (proxied by events.proxyEvent)
    → controller handles it (calls API, updates models)
```

`CommentListControl` and `PostEditSidebarControl` are event proxies. They forward events from child controls to the owning controller.

| Control | Dispatches | Listens To |
|---|---|---|
| CommentControl | `submit`, `score`, `delete` | Comment model: `change`, `changeScore` |
| CommentListControl | (proxies CommentControl events) | CommentList model: `add`, `remove` |
| PostReadonlySidebarControl | `favorite`, `unfavorite`, `score`, `fitModeChange` | Post model: `changeFavorite`, `changeScore` |
| PostEditSidebarControl | `change`, `submit`, `feature`, `merge`, `delete` | TagInputControl, PoolInputControl, PostNotesOverlayControl, Post model |
| PostContentControl | (none) | Post model: `changeContent` |
| PostNotesOverlayControl | `focus`, `blur`, `change` | Post model, note.polygon events |
| FileDropperControl | `fileadd`, `urladd`, `reset` | (none) |
| TagInputControl | `add`, `remove`, `change` | (none) |
| PoolInputControl | `add`, `remove`, `change` | (none) |
| CustomFieldInputControl | `change` | (none) |
| ExpanderControl | (none) | (none) |

## Autocomplete System

The autocomplete system has a base class and three specialized subclasses.

### AutoCompleteControl (Base Class)

Manages a dropdown suggestion list below or above a text input. Core features:
- Keyboard navigation: ArrowUp, ArrowDown, Enter, Escape, Tab, Shift+Tab, Delete
- Debounced search on keystroke (250ms)
- Dropdown positioning relative to input (above or below based on viewport position)
- Mouse hover and click selection
- Maximum result cap (default 15)
- `confirm` and `delete` callbacks for selection actions

**Keyboard shortcuts:**

| Key | Action |
|---|---|
| Escape | Hide dropdown |
| Tab | Select next suggestion |
| Shift+Tab | Select previous suggestion |
| ArrowUp | Select previous suggestion (wraps) |
| ArrowDown | Select next suggestion (wraps) |
| Enter | Confirm active suggestion |
| Delete | Delete active suggestion |

### TagAutoCompleteControl

Extends the base for tag name completion:
- Substring search for text >= 3 characters, prefix search for shorter text
- Results sorted by tag usage count descending
- Tags already in the current tag list show as `disabled`
- Negation support: when `isNegationAllowed` is true, `-term` searches non-prefixed and returns results with `-` prefix
- Caption includes tag category color CSS class, best-matching name, and post count
- Uses `TagList.search()` with fields: `names`, `category`, `usages`

### PoolAutoCompleteControl

Extends the base for pool name completion:
- Substring search for text >= 3 characters, prefix search for shorter text
- Results sorted by post count descending
- Caption includes pool category color CSS class, best-matching name, and post count
- Active suggestion returns the full Pool object (via `misc.matchingNames`)
- Uses `PoolList.search()` with fields: `id`, `names`, `category`, `postCount`, `version`

### CustomFieldAutoCompleteControl

Extends the base for custom field name completion:
- Fetches all custom fields once and filters client-side
- Loads field names into a module-level cache (`Set`) for validation
- Caption highlights matching text in `<b>` tags
- Exports `isValidFieldName()` and `ensureFieldNamesLoaded()` for use by `CustomFieldInputControl`

## Keyboard Shortcuts (Global)

| Control | Shortcut | Action |
|---|---|---|
| PostEditSidebarControl | `Ctrl+S` / `Cmd+S` | Submit the edit form |
| PostEditSidebarControl | `t` | Focus the tag input |
| TagInputControl | Enter / Space | Add input text as a tag |
| CustomFieldInputControl | Enter | Add or update custom field |
| PostNotesOverlayControl | Arrow keys | Move selected note (1px, or 10px with Ctrl) |
| PostNotesOverlayControl | Shift+Arrow | Scale selected note |
| PostNotesOverlayControl | Escape | Cancel note operation / remove last point |
| PostNotesOverlayControl | Enter | Finish drawing polygon |

### Keyboard Shortcut Suppression

The `PostNotesOverlayControl` calls `keyboard.pause()` during note interaction states (drawing, moving, scaling) to suppress global keyboard shortcuts like `q` (focus search). It calls `keyboard.unpause()` when returning to `ReadOnlyState`.

## Privilege System in Controls

Controls receive privilege flags from their constructors. They use these flags to show or hide UI elements, not to check authorization. The controller layer handles actual authorization.

### PostEditSidebarControl Privileges

This is the most privilege-aware control. It checks 17 privileges to conditionally show:

| Privilege | Controls Visibility Of |
|---|---|
| `post_edit_safety` | Safety radio buttons |
| `post_edit_info_dates` | Launch/media date inputs |
| `post_edit_source` | Source textarea |
| `post_edit_tag` | Tag input area |
| `post_edit_relation` | Relations input |
| `post_edit_note` | Note editing controls |
| `post_edit_flag` | Video flag checkboxes |
| `post_edit_content` | Content file dropper |
| `post_edit_thumbnail` | Thumbnail file dropper |
| `pool_edit_post` | Pool management area |
| `post_edit_custom_fields` | Custom field input |
| `post_create_anonymous` | Anonymous upload option |
| `post_delete` | Delete button |
| `post_feature` | Feature button |
| `post_merge` | Merge button |
| `post_edit_description` | Description textarea |
| `custom_field_list` | Custom field links |

## Settings Integration

Controls read user settings to customize behavior:

| Setting | Control | Effect |
|---|---|---|
| `fitMode` | PostContentControl | Default image fit mode (fit-both, fit-original, fit-width, fit-height) |
| `upscaleSmallPosts` | PostContentControl | Whether to enlarge images beyond native size |
| `autoplayVideos` | PostContentControl | Whether videos auto-play |
| `transparencyGrid` | PostContentControl | Whether to show a checkerboard behind transparent images |
| `tagSuggestions` | TagInputControl | Whether to show tag sibling suggestions |
| `keyboardShortcuts` | keyboard utility | Whether keyboard shortcuts are active globally |

## PostEditSidebarControl — Expander Sections

Creates 7 `ExpanderControl` instances that organize the edit form:

| Name | Title |
|---|---|
| `post-info` | "Basic info" |
| `post-tags` | "Tags (N)" |
| `post-notes` | "Notes" |
| `post-pools` | "Pools (N)" |
| `post-custom-fields` | "Custom Fields (N)" |
| `post-content` | "Content" |
| `post-management` | "Management" |

Each expander's collapsed/expanded state persists in `localStorage`. Expandable counts (tags, notes, pools, custom fields) update dynamically when items are added or removed.

## PostNotesOverlayControl — State Machine

The notes overlay uses a state machine pattern with 10 states:

```
ReadOnlyState
  └── can show note text on hover

PassiveState
  └── clickable → SelectedState

ReadyToDrawState
  ├── Shift+click → DrawingRectangleState
  └── click → DrawingPolygonState

SelectedState (from clicking a note)
  ├── drag → MovingNoteState
  ├── drag vertex → MovingPointState
  └── Shift+drag → ScalingNoteState

DrawingPolygonState
  ├── click → add vertex
  ├── Escape → remove last / cancel
  └── Enter → finish polygon

DrawingRectangleState
  └── mouseup → create rectangle note
```

All coordinates use unit space (0 to 1) with SVG `viewBox="0 0 1 1"`. Edge handles are fixed-size ellipses for consistent interaction regardless of image scale.
