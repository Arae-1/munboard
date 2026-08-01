# Post List Controller

**File:** `client/js/controllers/post_list_controller.js`
**Class:** `PostListController`

## Purpose

Controls the posts listing page (the main browse page). Shows a paginated, searchable list of posts with thumbnails. Supports bulk editing of tags, safety, and deletion.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `posts` | `PostListController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Creates a `PageController` instance.
2. Checks if the user has the `post_list` privilege.
3. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view posts.".
   - Returns early.
4. Stores the context as `this._ctx`.
5. Activates "posts" in the top navigation bar.
6. Sets the page title to "Listing posts".
7. Creates a `PostsHeaderView` with:
   - `hostNode`: The page controller's header holder node
   - `parameters`: The URL query parameters
   - `enableSafety`: `api.safetyEnabled()`
   - `canBulkEditTags`: `api.hasPrivilege("post_bulk_edit_tag")`
   - `canBulkEditSafety`: `api.hasPrivilege("post_bulk_edit_safety")`
   - `canBulkDelete`: `api.hasPrivilege("post_bulk_edit_delete")`
   - `bulkEdit.tags`: The `_bulkEditTags` getter
8. Registers a `navigate` event listener on the header view.
9. If the header view has a `_bulkDeleteEditor`, registers a `deleteSelectedPosts` event listener.
10. Initializes `this._postsMarkedForDeletion` as an empty array.
11. Calls `_syncPageController()`.

### Privileges Passed to the Header View

| Privilege | Context Property |
|---|---|
| `post_bulk_edit_tag` | `canBulkEditTags` |
| `post_bulk_edit_safety` | `canBulkEditSafety` |
| `post_bulk_edit_delete` | `canBulkDelete` |

---

## Public Methods

### `showSuccess(message)`
Delegates to `this._pageController.showSuccess(message)`.

---

## Properties

### `_bulkEditTags` (getter)
Parses the `tag` parameter from the URL query string. Splits by whitespace and filters empty strings. Returns an array of tag names used for bulk tag operations.

---

## Event Handlers

### `_evtNavigate(e)`
**Trigger:** User submits the header search/filter form.
**Parameters:** `e.detail.parameters`

**Code path:**
1. Updates the URL without dispatching via `router.showNoDispatch()`.
2. Merges the new parameters into `this._ctx.parameters`.
3. Calls `_syncPageController()`.

### `_evtTag(e)`
**Trigger:** User applies a bulk tag operation (add tag).
**Parameters:** `e.detail.post`

**Code path:**
1. Maps each bulk edit tag to `e.detail.post.tags.addByName(tag)` — all run in parallel via `Promise.all`.
2. Calls `e.detail.post.save()` after all tags are added.
3. On error: shows the error via `window.alert`.

### `_evtUntag(e)`
**Trigger:** User applies a bulk untag operation (remove tag).
**Parameters:** `e.detail.post`

**Code path:**
1. Iterates over each bulk edit tag and calls `e.detail.post.tags.removeByName(tag)`.
2. Calls `e.detail.post.save()`.
3. On error: shows the error via `window.alert`.

### `_evtChangeSafety(e)`
**Trigger:** User applies a bulk safety change.
**Parameters:** `e.detail.post`, `e.detail.safety`

**Code path:**
1. Sets `e.detail.post.safety` to `e.detail.safety`.
2. Calls `e.detail.post.save()`.
3. On error: shows the error via `window.alert`.

### `_evtMarkForDeletion(e)`
**Trigger:** User toggles a post for deletion.
**Parameters:** `e.detail` (the post selection event)

**Code path:**
1. If `e.detail.delete` is true:
   - Adds `e.detail.post` to `this._postsMarkedForDeletion`.
2. If `e.detail.delete` is false:
   - Removes the post from `this._postsMarkedForDeletion` by filtering on `id`.

### `_evtDeleteSelectedPosts(e)`
**Trigger:** User confirms bulk deletion.
**Code path:**
1. If `this._postsMarkedForDeletion` is empty, returns early.
2. Shows a browser `confirm()` dialog with the count of posts to delete.
3. If confirmed:
   - Runs `Promise.all` on each marked post's `delete()` method.
   - On success: clears the marked-for-deletion array and triggers navigation to refresh the page.
   - On error: shows the error via `window.alert`.

---

## Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `this._ctx.parameters` |
| `defaultLimit` | `settings.get().postsPerPage` (parsed as integer) |
| `getClientUrlForPage` | Returns client URL for "posts" with offset and limit |
| `requestPage` | Calls `PostList.search(query, offset, limit, fields)` |
| `pageRenderer` | Creates a `PostsPageView` with privilege context |

### Fields Requested

- `id`
- `thumbnailUrl`
- `type`
- `safety`
- `score`
- `favoriteCount`
- `commentCount`
- `tags`
- `version`

### Page Renderer Context

The page renderer adds these context properties to the view:

| Property | Source |
|---|---|
| `canViewPosts` | `api.hasPrivilege("post_view")` |
| `canBulkEditTags` | `api.hasPrivilege("post_bulk_edit_tag")` |
| `canBulkEditSafety` | `api.hasPrivilege("post_bulk_edit_safety")` |
| `canBulkDelete` | `api.hasPrivilege("post_bulk_edit_delete")` |
| `bulkEdit.tags` | `this._bulkEditTags` |
| `bulkEdit.markedForDeletion` | `this._postsMarkedForDeletion` |
| `postFlow` | `settings.get().postFlow` |

### View Event Listeners (per page)

Each page view gets these listeners:
- `tag` — `_evtTag`
- `untag` — `_evtUntag`
- `changeSafety` — `_evtChangeSafety`
- `markForDeletion` — `_evtMarkForDeletion`

---

## API Calls

| Call | Purpose |
|---|---|
| `PostList.search(query, offset, limit, fields)` | Searches posts |
| `post.save()` | Saves post after bulk edit |
| `post.delete()` | Deletes a post (bulk) |
| `post.tags.addByName(tag)` | Adds a tag by name |
| `post.tags.removeByName(tag)` | Removes a tag by name |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing (showNoDispatch) |
| `api` | Privilege checks |
| `settings` | Posts per page and post flow settings |
| `uri` | Client link formatting |
| `PostList` | Searches posts |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `PostsHeaderView` | Renders the search/create header |
| `PostsPageView` | Renders the post grid |
| `EmptyView` | Error view when missing privileges |
