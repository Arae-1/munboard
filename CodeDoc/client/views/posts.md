---
title: Post Views
type: summary
created: 2026-08-01
updated: 2026-08-01
tags: [client, views, posts]
---

# Post Views

## PostsPageView (`js/views/posts_page_view.js`)

**Template**: `posts-page`

**Purpose**: Renders a list of post thumbnails with bulk editing overlays.

**Constructor** (`ctx`):
- `ctx.hostNode` - node to render into
- `ctx.response.results` - array of Post models
- `ctx.bulkEdit` - bulk edit state (tags, markedForDeletion)

**Render method**: Calls `views.replaceContent` with the `posts-page` template. Iterates over post results, builds a map of post ID to post, and a map of post ID to list item DOM node. Attaches click handlers to tag flipper, safety flipper, and delete flipper elements within each list item.

**Events handled**:
- Post `change` event — updates list item display and syncs bulk editor highlights
- Tag flipper click — dispatches `tag` or `untag` custom event
- Safety flipper click — dispatches `changeSafety` custom event
- Delete flipper click — dispatches `markForDeletion` custom event

**Key methods**:
- `_syncBulkEditorsHighlights()` — updates CSS classes on all list items based on current bulk edit state

---

## PostsHeaderView (`js/views/posts_header_view.js`)

**Template**: `posts-header`

**Purpose**: Renders the post list header with search form, safety filter buttons, and bulk edit controls.

**Constructor** (`ctx`):
- `ctx.settings` — user browsing settings
- `ctx.parameters` — URL query parameters
- `ctx.canBulkEditTags`, `ctx.canBulkEditSafety`, `ctx.canBulkDelete` — permission flags

**Render method**: Renders template, initializes TagAutoCompleteControl on the search input, binds safety button clicks, and conditionally creates BulkTagEditor, BulkSafetyEditor, and BulkDeleteEditor instances.

**Inner classes**:

### BulkEditor (base class)
Abstract base with `toggleOpen` and `toggleHide` methods. Manages open/close link nodes. Subclasses override `_evtOpenLinkClick` and `_evtCloseLinkClick`.

### BulkSafetyEditor extends BulkEditor
Opens/closes safety editing mode. Enables exit confirmation when open.

### BulkTagEditor extends BulkEditor
Contains a TagAutoCompleteControl for tag input. Handles form submit. Has `focus()`, `blur()`, and `value` getter.

### BulkDeleteEditor extends BulkEditor
Handles delete form submission. Dispatches `deleteSelectedPosts` custom event.

**Events dispatched**:
- `navigate` — on search form submit, safety toggle, or bulk editor changes; carries new URL parameters
- `changeSafety` — proxied from safety buttons
- `deleteSelectedPosts` — on bulk delete submit

---

## PostUploadView (`js/views/post_upload_view.js`)

**Templates**: `post-upload` (main), `post-upload-row` (per file row)

**Purpose**: File upload form with drag-and-drop, URL paste, and lookalike detection.

**Constructor** (`ctx`):
- Renders into `#content-holder`
- Creates a FileDropperControl for the content input

**Inner classes**:

### Uploadable (base)
Properties: `lookalikes`, `safety`, `flags`, `tags`, `relations`, `anonymous`.

### File extends Uploadable
Creates preview URL from browser File object via `URL.createObjectURL`.

### Url extends Uploadable
Determines MIME type from URL file extension.

**Key methods**:
- `addUploadables(uploadables)` — adds files/URLs to the upload list, skips duplicates
- `removeUploadable(uploadable)` — removes a file/URL from the list
- `enableForm()` / `disableForm()` — enables/disables form and shows/hides cancel button

**Events dispatched**:
- `submit` — on form submit, carries `uploadables`, `skipDuplicates`, `alwaysUploadSimilar`, `pauseRemainOnError`
- `cancel` — on cancel button click

---

## PostMainView (`js/views/post_main_view.js`)

**Template**: `post-main`

**Purpose**: Main post view with content display, sidebar, description, and comments.

**Constructor** (`ctx`):
- `ctx.post` — the Post model
- `ctx.editMode` — boolean
- `ctx.prevPostId`, `ctx.nextPostId` — navigation IDs

**Render method**: Renders template, creates PostContentControl for the post content, PostNotesOverlayControl for annotations, and installs sidebar (PostEditSidebarControl or PostReadonlySidebarControl based on editMode). Installs CommentControl for comment form and CommentListControl for comments.

**Keyboard bindings**:
- `e` — toggle edit mode
- `a` / `left` — previous post
- `d` / `right` — next post
- `del` — delete post (edit mode only)

**Touch**: Swipe left/right for previous/next post navigation.

---

## PostDetailView (`js/views/post_detail_view.js`)

**Template**: `post-detail`

**Purpose**: Detail view for a single post with summary and merge sections.

**Constructor** (`ctx`):
- `ctx.post` — Post model
- `ctx.section` — `'summary'` or `'merge'`

**Render method**: Renders template with section tab navigation. For `merge` section, creates PostMergeView. Highlights active tab.

**Delegates to inner view**: `clearMessages`, `enableForm`, `disableForm`, `showSuccess`, `showError`, `selectPost`.

---

## PostMergeView (`js/views/post_merge_view.js`)

**Templates**: `post-merge`, `post-merge-side`

**Purpose**: Merge two posts by selecting a left post (source) and right post (target).

**Constructor** (`ctx`):
- `ctx.post` — the source post (left side)
- Right side is initially empty and editable

**Render method**: Renders template with left and right side containers. Right side has an editable text input and search button.

**Events dispatched**:
- `submit` — on form submit with `post`, `targetPost`, `useOldContent`
- `select` — on right-side post ID entry (Enter key or Search button)

---

## PostMergeView Side Templates

### Template: `post-merge`
Main merge form with two side-by-side post panels, a confirmation checkbox, and a merge submit button.

### Template: `post-merge-side`
Displays a post thumbnail, target post selector (radio), and target post content selector (radio). Shows the post ID, uploader, creation time, file size, and MIME type.
