# Post Main Controller

**File:** `client/js/controllers/post_main_controller.js`
**Class:** `PostMainController extends BasePostController`

## Purpose

Controls the main post view page and the post edit page. This is the primary controller for viewing and editing a single post. It handles:
- Post content display
- Post editing (tags, safety, description, source, relations, content replacement, thumbnail, launch date, media created date, custom fields, flags)
- Post scoring, favoriting, featuring
- Post deletion and merge navigation
- Comment creation, editing, scoring, and deletion
- Previous/next post navigation with preloading
- Query parameter cleanup (strip query from URL, save to history state)

## Routing

| Route | Controller | Edit Mode |
|---|---|---|
| `post/:id` | `PostMainController` | `false` |
| `post/:id/edit` | `PostMainController` | `true` |

### Edit Route Special Behavior

The edit route (`/post/:id/edit`) checks the `post_edit` privilege. If the user lacks this privilege, the controller redirects to the non-edit view page with the query parameter preserved.

The edit route also restores URL parameters from `ctx.state.parameters` (history state). This allows query parameters to survive browser history navigation.

---

## Constructor

**Parameters:** `ctx` (the router context), `editMode` (boolean)

**Code path:**
1. Calls `super(ctx)` — initializes `BasePostController`.
2. Extracts `parameters` from `ctx.parameters`.
3. Calls `Promise.all([Post.get(id), PostList.getAround(id, query)])` to fetch the post and its neighbors in parallel.

**Query parameter cleanup:**
- If `parameters.query` exists, the controller:
  1. Saves the parameters to `ctx.state.parameters` (history state).
  2. Replaces the URL with a clean URL (no query string) using `router.replace()`.
  3. This keeps the URL clean while preserving the query for back/forward navigation.

**Success path:**
- Extracts `prevPostId` and `nextPostId` from the around response.
- Preloads nearby posts: for each neighbor, calls `Post.get(id, {noProgress: true}).then(misc.preloadPostImages)` and `PostList.getAround(id, query, {noProgress: true})`.
- Creates a `PostMainView` with:
  - `post`: The post model
  - `editMode`: Whether editing is active
  - `prevPostId` / `nextPostId`: Neighbor IDs
  - Privileges for editing, deleting, featuring, commenting, custom fields
  - `parameters`: The URL parameters

**Error path:**
- Creates an `EmptyView`.
- Shows the error message.

### Post Query Cleanup

When a post URL contains a query string (from search results), the controller:
1. Preserves the full parameters in browser history state (`ctx.state.parameters`).
2. Replaces the current URL with a clean `/post/:id` or `/post/:id/edit` URL.
3. This uses `router.replace(url, ctx.state, false)` where `false` means no dispatch.

### Nearby Post Preloading

For each neighbor (previous and next), the controller:
- Fetches the post with `{noProgress: true}` (no progress indicator)
- Preloads the post images via `misc.preloadPostImages`
- Fetches the around data via `PostList.getAround(id, query, {noProgress: true})`

### Privileges Passed to the View

| Privilege | Context Property |
|---|---|
| `post_edit` | `canEditPosts` |
| `post_edit_description` | `canEditPostDescription` |
| `post_edit_custom_fields` | `canEditPostCustomFields` |
| `custom_field_list` | `canViewCustomFields` |
| `post_delete` | `canDeletePosts` |
| `post_feature` | `canFeaturePosts` |
| `comment_list` | `canListComments` |
| `comment_create` | `canCreateComments` |

---

## Event Handlers

### View Component Setup

The controller conditionally sets up event listeners on three view components:
- `sidebarControl` — post actions (favorite, score, edit, delete, etc.)
- `commentControl` — comment creation form
- `commentListControl` — existing comments (edit, score, delete)
- `postDescription` — description field changes

### Post Actions (sidebarControl)

#### `_evtFavoritePost(e)`
**Trigger:** User favorites a post.
**Privilege:** `post_favorite`
**Code path:** Calls `e.detail.post.addToFavorites()`. On error: `window.alert`.

#### `_evtUnfavoritePost(e)`
**Trigger:** User unfavorites a post.
**Privilege:** `post_favorite`
**Code path:** Calls `e.detail.post.removeFromFavorites()`. On error: `window.alert`.

#### `_evtScorePost(e)`
**Trigger:** User sets a score on a post.
**Privilege:** `post_score`
**Code path:** Calls `e.detail.post.setScore(e.detail.score)`. On error: `window.alert`.

#### `_evtFitModeChange(e)`
**Trigger:** User changes the fit mode (image display mode).
**Code path:**
1. Gets the current browsing settings.
2. Sets `browsingSettings.fitMode` to `e.detail.mode`.
3. Saves settings via `settings.save(browsingSettings)`.

#### `_evtPostChange(e)`
**Trigger:** Any change to the post form.
**Code path:** Calls `misc.enableExitConfirmation()`.

#### `_evtUpdatePost(e)`
**Trigger:** User submits the post edit form.
**Parameters:** `e.detail.post`, `e.detail.description`, `e.detail.safety`, `e.detail.flags`, `e.detail.relations`, `e.detail.content`, `e.detail.thumbnail`, `e.detail.source`, `e.detail.launchDate`, `e.detail.mediaCreatedDate`

**Code path:**
1. Disables the sidebar form.
2. Clears messages.
3. Updates post properties (only if not `undefined`/`null`, except `launchDate` and `mediaCreatedDate` which check only `!== undefined`):
   - `description` — sets `post.description`
   - `safety` — sets `post.safety`
   - `flags` — sets `post.flags`
   - `relations` — sets `post.relations`
   - `content` — sets `post.newContent`
   - `thumbnail` — sets `post.newThumbnail`
   - `source` — sets `post.source`
   - `launchDate` — sets `post.launchDate`
   - `mediaCreatedDate` — sets `post.mediaCreatedDate`
4. Calls `post.save()`.

**Note:** `description` is set twice in the code (first at line 195, again at line 222). This is a bug — the second assignment overwrites the first. However, since both check `e.detail.description`, the net effect is the same.

**Success path:**
- Shows "Post saved.".
- Enables the form.
- Disables exit confirmation.

**Error path:**
- Shows the error message.
- Enables the form.

#### `_evtFeaturePost(e)`
**Trigger:** User features the post.
**Code path:**
1. Disables the sidebar form.
2. Clears messages.
3. Calls `e.detail.post.feature()`.

**Success path:** Shows "Post featured.", enables form.
**Error path:** Shows error message, enables form.

#### `_evtDeletePost(e)`
**Trigger:** User deletes the post.
**Code path:**
1. Disables the sidebar form.
2. Clears messages.
3. Calls `e.detail.post.delete()`.

**Success path:**
- Disables exit confirmation.
- Routes to the posts list page.
- Shows "Post deleted.".

**Error path:**
- Shows error message.
- Enables form.

#### `_evtMergePost(e)`
**Trigger:** User navigates to the merge page.
**Code path:** Routes to `router.show(uri.formatClientLink("post", post.id, "merge"))`.

### Comment Actions

#### `_evtCommentChange(e)`
**Trigger:** Any change to the comment form.
**Code path:** Calls `misc.enableExitConfirmation()`.

#### `_evtCreateComment(e)`
**Trigger:** User submits a new comment.
**Code path:**
1. Disables the comment form.
2. Creates a new comment via `Comment.create(this._post.id)`.
3. Sets `comment.text` to `e.detail.text`.
4. Calls `comment.save()`.

**Success path:**
- Adds the comment to `this._post.comments`.
- Exits edit mode on the comment control.
- Enables the form.
- Disables exit confirmation.

**Error path:**
- Shows error message.
- Enables the form.

#### `_evtUpdateComment(e)`
**Trigger:** User edits an existing comment.
**Code path:**
1. Sets `e.detail.comment.text` to `e.detail.text`.
2. Calls `e.detail.comment.save()`.
3. On error: shows error on `e.detail.target`.

#### `_evtScoreComment(e)`
**Trigger:** User scores a comment.
**Code path:** Calls `e.detail.comment.setScore(e.detail.score)`. On error: `window.alert`.

#### `_evtDeleteComment(e)`
**Trigger:** User deletes a comment.
**Code path:** Calls `e.detail.comment.delete()`. On error: `window.alert`.

---

## API Calls

| Call | Purpose |
|---|---|
| `Post.get(id)` | Fetches the post |
| `PostList.getAround(id, query)` | Fetches previous/next posts |
| `Post.get(neighborId, {noProgress: true})` | Preloads a neighbor post (silently) |
| `PostList.getAround(neighborId, query, {noProgress: true})` | Preloads around data for a neighbor (silently) |
| `post.save()` | Saves post edits |
| `post.feature()` | Features the post |
| `post.delete()` | Deletes the post |
| `post.setScore(score)` | Sets the score |
| `post.addToFavorites()` | Adds to favorites |
| `post.removeFromFavorites()` | Removes from favorites |
| `Comment.create(postId)` | Creates a new comment instance |
| `comment.save()` | Saves a new/edited comment |
| `comment.setScore(score)` | Sets a comment score |
| `comment.delete()` | Deletes a comment |
| `settings.save(browsingSettings)` | Saves browsing settings (fit mode) |

---

## Dependencies

| Module | Use |
|---|---|
| `router` | Client-side routing and URL replacement |
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `misc` | Exit confirmation and image preloading |
| `settings` | Browsing settings (fit mode) |
| `Comment` | Comment model |
| `Post` | Post model |
| `PostList` | Post search and around navigation |
| `PostMainView` | Renders the post page |
| `BasePostController` | Base class with privilege check |
| `EmptyView` | Error view |
