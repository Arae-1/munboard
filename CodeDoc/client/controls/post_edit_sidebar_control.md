# PostEditSidebarControl

## UI Element

Manages the entire post edit sidebar form. This is the most complex sidebar control. It coordinates:
- Tag input with autocomplete and sibling suggestions
- Pool input with autocomplete
- Custom field input with autocomplete and color selection
- Note overlay drawing (polygon or rectangle)
- File droppers for post content and thumbnail replacement
- Safety radio buttons, video flags (loop/sound)
- Relations input, source input, date inputs, description textarea
- Post management actions (feature, merge, delete)
- Note copy/paste to and from clipboard
- Expander sections for organizing the form

Uses the `post-edit-sidebar` template.

## Constructor

### `constructor(hostNode, post, postContentControl, postNotesOverlayControl)`
- `hostNode`: The sidebar DOM container.
- `post`: The `Post` model.
- `postContentControl`: A `PostContentControl` instance.
- `postNotesOverlayControl`: A `PostNotesOverlayControl` instance.

Sets the notes overlay to passive edit mode. Renders the template with comprehensive privilege checks (see privilege list below).

### Privileges Checked

| Privilege | Controls Visibility Of |
|---|---|
| `post_edit_safety` | Safety radio buttons |
| `post_edit_info_dates` | Launch/media date inputs |
| `post_edit_source` | Source textarea |
| `post_edit_tag` | Tag input |
| `post_edit_relation` | Relations input |
| `post_edit_note` | Note editing controls (hidden for video) |
| `post_edit_flag` | Video flag checkboxes |
| `post_edit_content` | Content file dropper |
| `post_edit_thumbnail` | Thumbnail file dropper |
| `pool_edit_post` | Pool management |
| `post_edit_custom_fields` | Custom field input |
| `post_create_anonymous` | Anonymous creation option |
| `post_delete` | Delete button |
| `post_feature` | Feature button |
| `post_merge` | Merge button |
| `post_edit_description` | Description textarea |
| `user_view` | User names |
| `tag_view` | Tag links |
| `custom_field_list` | Custom field links |

### Expander Sections

Creates 6 `ExpanderControl` instances:

| Name | Title | Content Selectors |
|---|---|---|
| `post-info` | "Basic info" | `.safety, .relations, .flags, .post-source, .post-launch-date, .post-media-date` |
| `post-tags` | `Tags (N)` | `.tags` |
| `post-notes` | "Notes" | `.notes` |
| `post-pools` | `Pools (N)` | `.pools` |
| `post-custom-fields` | `Custom Fields (N)` | `.custom-fields` |
| `post-content` | "Content" | `.post-content, .post-thumbnail` |
| `post-management` | "Management" | `.management` |

Stores references to `_tagsExpander`, `_notesExpander`, `_poolsExpander`, and `_customFieldsExpander` for dynamic title updates.

### Sub-Controls Created

1. **TagInputControl** bound to `.tags input`, using `post.tags`.
2. **PoolInputControl** bound to `.pools input`, using `post.pools`.
3. **CustomFieldInputControl** bound to `.custom-fields input`, using `post.customFields` and `post.customFieldColors`.
4. **FileDropperControl** for content (`.post-content .dropper-container`): allows URLs, locks after selection.
5. **FileDropperControl** for thumbnail (`.post-thumbnail .dropper-container`): locked after selection.

### Event Bindings (overall)

| Event | Source | Handler |
|---|---|---|
| `submit` | Form | `_evtSubmit` |
| `fileadd` | Content file dropper | Stores the file as `_newPostContent` |
| `urladd` | Content file dropper | Stores the URL as `_newPostContent` |
| `fileadd` | Thumbnail file dropper | Stores the file as `_newPostThumbnail`, shows removal link |
| `click` | Thumbnail removal link | `_evtRemoveThumbnailClick` |
| `click` | Add note link | `_evtAddNoteClick` |
| `click` | Copy notes link | `_evtCopyNotesClick` |
| `click` | Paste notes link | `_evtPasteNotesClick` |
| `click` | Delete note link | `_evtDeleteNoteClick` |
| `click` | Feature link | `_evtFeatureClick` |
| `click` | Merge link | `_evtMergeClick` |
| `click` | Delete link | `_evtDeleteClick` |
| `blur` | Notes overlay | `_evtNoteBlur` |
| `focus` | Notes overlay | `_evtNoteFocus` |
| `changeContent` | Post model | `_evtPostContentChange` |
| `changeThumbnail` | Post model | `_evtPostThumbnailChange` |
| `change` | All form inputs | Dispatches a `change` event upward |
| `change` | Notes overlay | Dispatches a `change` event upward |
| `add` / `remove` | Post notes/pools models | `_syncExpanderTitles` |
| `change` | Tag control | Dispatches `change`, `_syncExpanderTitles` |
| `change` | Pool control | Dispatches `change`, `_syncExpanderTitles` |
| `change` | Note textarea | `_evtNoteTextChangeRequest` |

## Public Methods

### `enableForm()`
Enables all form controls.

### `disableForm()`
Disables all form controls.

### `clearMessages()`
Clears success and error messages from the host node.

### `showSuccess(message)`
Displays a success message in the host node.

### `showError(message)`
Displays an error message in the host node.

## Private Methods

### `_syncExpanderTitles()`
Updates the title text of the notes, tags, pools, and custom fields expanders to reflect current counts.

### `_evtPostContentChange(e)`
Resets the content file dropper.

### `_evtPostThumbnailChange(e)`
Resets the thumbnail file dropper.

### `_evtRemoveThumbnailClick(e)`
Resets the thumbnail dropper. Clears `_newPostThumbnail`. Hides the removal link.

### `_evtFeatureClick(e)`
Shows a confirmation dialog. If confirmed, dispatches a `feature` CustomEvent with the post.

### `_evtMergeClick(e)`
Dispatches a `merge` CustomEvent with the post.

### `_evtDeleteClick(e)`
Shows a confirmation dialog. If confirmed, dispatches a `delete` CustomEvent with the post.

### `_evtNoteTextChangeRequest(e)`
If a note is being edited, updates its text from the note textarea.

### `_evtNoteFocus(e)`
Stores the focused note as `_editedNote`. Enables the add-note link, enables the delete-note link, enables the note textarea, and populates the textarea with the note's text.

### `_evtNoteBlur(e)`
Commits note text changes. Disables the delete-note link. Blurs and disables the note textarea. Clears the textarea value.

### `_evtAddNoteClick(e)`
If the target has the `inactive` class, returns. Otherwise, adds the `inactive` class and switches the notes overlay to drawing mode.

### `_evtCopyNotesClick(e)`
Serializes all notes to JSON (polygon as `[[x,y],...]` arrays, text as string). Creates a hidden textarea, copies to clipboard via `document.execCommand("copy")`. Shows success or failure alert.

### `_evtPasteNotesClick(e)`
Shows a prompt for JSON input. Parses the JSON. Clears existing notes. Recreates `Note` and `Point` model instances. Adds them to the post's notes collection.

### `_evtDeleteNoteClick(e)`
If the target has the `inactive` class, returns. Removes the currently editing note from the model. Switches the notes overlay to passive edit mode.

### `_evtSubmit(e)`
Prevents default form submission. Dispatches a `submit` CustomEvent with a detailed object containing:
- `post`: the Post model
- `safety`: selected safety value (lowercased)
- `flags`: array of video flags from checkbox state
- `tags`: space-split tag names from input
- `pools`: space-split pool names from input
- `relations`: space-split relation IDs parsed as integers
- `content`: the new content file or URL
- `thumbnail`: the new thumbnail file (or undefined if null)
- `source`: the source textarea value
- `launchDate`: formatted RFC 3339 date string
- `mediaCreatedDate`: formatted RFC 3339 date string
- `description`: description textarea value (if user has description edit privilege)

## DOM Getters (querySelector-based)

| Getter | Returned Element |
|---|---|
| `_formNode` | `form` |
| `_submitButtonNode` | `.submit` |
| `_safetyButtonNodes` | `.safety input` (all) |
| `_tagInputNode` | `.tags input` |
| `_poolInputNode` | `.pools input` |
| `_customFieldInputNode` | `.custom-fields input` |
| `_loopVideoInputNode` | `.flags input[name=loop]` |
| `_soundVideoInputNode` | `.flags input[name=sound]` |
| `_relationsInputNode` | `.relations input` |
| `_contentInputNode` | `.post-content .dropper-container` |
| `_thumbnailInputNode` | `.post-thumbnail .dropper-container` |
| `_thumbnailRemovalLinkNode` | `.post-thumbnail a` |
| `_sourceInputNode` | `.post-source textarea` |
| `_launchDateInputNode` | `.post-launch-date input` |
| `_mediaDateInputNode` | `.post-media-date input` |
| `_featureLinkNode` | `.management .feature` |
| `_mergeLinkNode` | `.management .merge` |
| `_deleteLinkNode` | `.management .delete` |
| `_addNoteLinkNode` | `.notes .add` |
| `_copyNotesLinkNode` | `.notes .copy` |
| `_pasteNotesLinkNode` | `.notes .paste` |
| `_deleteNoteLinkNode` | `.notes .delete` |
| `_noteTextareaNode` | `.notes textarea` |
| `_descriptionTextareaNode` | `textarea#post-description` (document-wide) |

## Communication

- **Dispatches**: `change`, `submit`, `feature`, `merge`, `delete` CustomEvents.
- **Listens to**: events from `TagInputControl`, `PoolInputControl`, `PostNotesOverlayControl`, `FileDropperControl`, and the Post model.
- **Coordinates**: all sub-controls and the notes overlay.

## State Management

- `_newPostContent`: file or URL for the replacement content.
- `_newPostThumbnail`: file for the replacement thumbnail.
- `_editedNote`: the `Note` model currently focused for editing.

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+S` / `Cmd+S` | Submits the form |
| `t` | Focuses the tag input |

The `Ctrl+S` shortcut is bound both globally and on the tag input element to prevent the browser save dialog.
