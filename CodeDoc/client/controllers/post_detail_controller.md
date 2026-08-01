# Post Detail Controller

**File:** `client/js/controllers/post_detail_controller.js`
**Class:** `PostDetailController extends BasePostController`

## Purpose

Controls the post detail page for non-main sections (currently only the merge section). Handles post selection, merge operations, and ID-change detection.

## Routing

| Route | Controller | Section |
|---|---|---|
| `post/:id/merge` | `PostDetailController` | `"merge"` |

---

## Constructor

**Parameters:** `ctx` (the router context), `section` (`"merge"`)

**Code path:**
1. Calls `super(ctx)` — initializes `BasePostController`:
   - Checks `post_view` privilege.
   - Activates "posts" in the navigation.
   - Sets the page title.
2. Calls `Post.get(ctx.parameters.id)` to fetch the post.

**Success path:**
- Stores the original post ID in `this._id`.
- Registers a `change` event listener on the post model for `_evtSaved`.
- Calls `_installView(post, section)`.

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

---

## Public Methods

### `showSuccess(message)`
**Parameters:** `message` (string)
**Code path:** Calls `this._view.showSuccess(message)`.

---

## Private Methods

### `_installView(post, section)`
**Parameters:** `post` (Post model), `section` (string)

**Code path:**
1. Creates a `PostDetailView` with:
   - `post`: The post model
   - `section`: The current section
   - `canMerge`: `api.hasPrivilege("post_merge")`
2. Registers two event listeners:
   - `select` — calls `_evtSelect`
   - `merge` — calls `_evtMerge`

---

## Event Handlers

### `_evtSelect(e)`
**Trigger:** User selects a post (for merge target).
**Parameters:** `e.detail.postId`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `Post.get(e.detail.postId)`.

**Success path:**
- Selects the post on the view via `this._view.selectPost(post)`.
- Enables the form.

**Error path:**
- Shows the error message.
- Enables the form.

### `_evtSaved(e, section)`
**Trigger:** The post model fires a `change` event after a save.
**Parameters:** `e.detail.post`

**Code path:**
1. Calls `misc.disableExitConfirmation()`.
2. If the post ID changed from the original, calls `router.replace()` to update the URL.

### `_evtMerge(e)`
**Trigger:** User confirms a post merge.
**Parameters:** `e.detail.post`, `e.detail.targetPost.id`, `e.detail.useOldContent`

**Code path:**
1. Clears all messages.
2. Disables the form.
3. Calls `e.detail.post.merge(targetPostId, useOldContent)`.

**Success path:**
- Re-installs the view for the new post.
- Shows "Post merged.".
- Replaces the URL with the target post's merge page.

**Error path:**
- Shows the error message.
- Enables the form.

---

## API Calls

| Call | Purpose |
|---|---|
| `Post.get(id)` | Fetches a single post by ID |
| `post.merge(targetPostId, useOldContent)` | Merges the post into another post |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | Privilege checks |
| `misc` | Exit confirmation management |
| `uri` | Client link formatting |
| `settings` | (Imported but not used directly) |
| `Post` | Post model |
| `PostList` | (Imported but not used directly) |
| `PostDetailView` | Renders the post detail/merge page |
| `BasePostController` | Base class with privilege check and navigation setup |
| `EmptyView` | Error view |
