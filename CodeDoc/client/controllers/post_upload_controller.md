# Post Upload Controller

**File:** `client/js/controllers/post_upload_controller.js`
**Class:** `PostUploadController`

## Purpose

Controls the post upload page. Manages the upload queue, reverse image search for duplicate detection, and the sequential upload of posts.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `upload` | `PostUploadController` | Constructor |

---

## Constructor

**Code path:**
1. Initializes `this._lastCancellablePromise` to `null`.
2. Checks if the user has the `post_create` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to upload posts.".
   - Returns early.
4. Activates "upload" in the top navigation bar.
5. Sets the page title to "Upload".
6. Creates a `PostUploadView` with:
   - `canUploadAnonymously`: `api.hasPrivilege("post_create_anonymous")`
   - `canViewPosts`: `api.hasPrivilege("post_view")`
   - `enableSafety`: `api.safetyEnabled()`
7. Registers three event listeners: `change`, `submit`, `cancel`.

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `post_create_anonymous` | `canUploadAnonymously` |
| `post_view` | `canViewPosts` |

---

## Event Handlers

### `_evtChange(e)`
**Trigger:** The uploadable list changes (files added or removed).
**Parameters:** `e.detail.uploadables` (array of uploadable items)

**Code path:**
1. If `e.detail.uploadables.length` is non-zero:
   - Calls `misc.enableExitConfirmation()`.
2. If the uploadable list is empty:
   - Calls `misc.disableExitConfirmation()`.
   - Clears all messages on the view.

### `_evtCancel(e)`
**Trigger:** User clicks the cancel button.
**Code path:** Calls `this._lastCancellablePromise.abort()` to abort the current upload request.

### `_evtSubmit(e)`
**Trigger:** User submits the upload form.
**Parameters:** `e.detail.uploadables` (array), `e.detail.skipDuplicates` (boolean), `e.detail.alwaysUploadSimilar` (boolean), `e.detail.pauseRemainOnError` (boolean)

**Code path:**
1. Disables the upload form.
2. Clears all messages.
3. Initializes `anyFailures` to `false`.
4. Processes each uploadable sequentially using `Array.reduce`:
   - Creates a promise chain where each step calls `_uploadSinglePost()`.
   - If `_uploadSinglePost` rejects:
     - Sets `anyFailures = true`.
     - If the error has an `uploadable` property:
       - If the error has `similarPosts`:
         - Stores lookalikes on the uploadable.
         - Updates the uploadable in the view.
         - Shows an info message with the uploadable reference.
       - Else:
         - Shows the error message with the uploadable reference.
     - If the error has no `uploadable` property:
       - Shows the error with the original uploadable.
     - If `e.detail.pauseRemainOnError` is true:
       - Rejects the promise chain (stops processing).

**After all uploadables:**
- If `anyFailures` is true:
  - Rejects the final promise.
- If `anyFailures` is false (all succeeded):
  - Clears messages.
  - Disables exit confirmation.
  - Routes to the posts list page.
  - Shows "Posts uploaded.".

**Error fallback (final):**
- Shows the generic error message: "One or more posts needs your attention; click 'resume upload' when you're ready."
- Enables the form.

---

## Private Methods

### `_uploadSinglePost(uploadable, skipDuplicates, alwaysUploadSimilar)`
**Parameters:**
- `uploadable` (object with `url`, `file`, `lookalikesConfirmed`, `anonymous`, `safety`, `flags`, `tags`, `relations`)
- `skipDuplicates` (boolean)
- `alwaysUploadSimilar` (boolean)

**Returns:** A Promise

**Code path:**
1. Starts the progress indicator.
2. If `uploadable.lookalikesConfirmed` is not set:
   - Calls `Post.reverseSearch(uploadable.url || uploadable.file)` to check for duplicates.
   - Stores the promise as `this._lastCancellablePromise` (so it can be aborted).
3. If `uploadable.lookalikesConfirmed` is set:
   - Sets the reverse search promise to `Promise.resolve()` (skips search).

**On reverse search result:**
- If a result exists:
  - If `searchResult.exactPost` exists (exact duplicate):
    - If `skipDuplicates` is true:
      - Removes the uploadable from the view.
      - Resolves the promise gracefully.
    - If `skipDuplicates` is false:
      - Rejects with an error: "Post already uploaded (@{id})". Sets `error.uploadable`.
  - If `searchResult.similarPosts` has items and `alwaysUploadSimilar` is false:
    - Rejects with an error: "Found {n} similar posts. You can resume or discard this upload.". Sets `error.uploadable` and `error.similarPosts`.
- If no issues, proceeds to save:
  - Converts the uploadable to a `Post` model via `_uploadableToPost()`.
  - Saves the post via `post.save(uploadable.anonymous)`.
  - On success: removes the uploadable from the view.
  - Stores the save promise as `this._lastCancellablePromise`.

**Finally:**
- On success: calls `progress.done()`.
- On error: sets `error.uploadable = uploadable`, calls `progress.done()`, rejects.

### `_uploadableToPost(uploadable)`
**Parameters:** `uploadable` (object)
**Returns:** A `Post` model instance

**Code path:**
1. Creates a new `Post` instance.
2. Sets `post.safety` and `post.flags`.
3. For each tag name in `uploadable.tags`, creates a `Tag` model with `names = [tagName]` and adds it to `post.tags`.
4. Sets `post.relations`.
5. Sets `post.newContent` to `uploadable.url || uploadable.file`.
6. If `uploadable.url` exists, sets `post.source` to `uploadable.url`.

---

## API Calls

| Call | Purpose |
|---|---|
| `Post.reverseSearch(urlOrFile)` | Checks if a file/URL is a duplicate or similar to existing posts |
| `post.save(anonymous)` | Saves the new post |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `router` | Client-side routing |
| `uri` | Client link formatting |
| `misc` | Exit confirmation management |
| `progress` | Progress indicator (start/done) |
| `top_navigation` | Navigation bar activation and title |
| `Post` | Post model |
| `Tag` | Tag model |
| `PostUploadView` | Renders the upload page |
| `EmptyView` | Error view when missing privileges |
