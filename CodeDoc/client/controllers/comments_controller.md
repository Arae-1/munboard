# Comments Controller

**File:** `client/js/controllers/comments_controller.js`
**Class:** `CommentsController`

## Purpose

Controls the comments listing page. Shows a paginated list of the latest comments across all posts. Supports comment editing, scoring, and deletion.

## Routing

| Route | Controller | Handler |
|---|---|---|
| `comments` | `CommentsController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Checks if the user has the `comment_list` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view comments.".
   - Returns early.
3. Activates the "comments" item in the top navigation bar.
4. Sets the page title to "Listing comments".
5. Creates a `PageController` instance.
6. Runs the page controller with the following configuration:

### Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `ctx.parameters` (query string parameters) |
| `defaultLimit` | `10` items per page |
| `getClientUrlForPage` | Returns the client URL for the "comments" route with offset and limit |
| `requestPage` | Calls `PostList.search()` with query `"sort:comment-date comment-count:1.."` and fields: `id`, `comments`, `commentCount`, `thumbnailUrl` |
| `pageRenderer` | Creates a `CommentsPageView` with privileges set |

### Privileges Passed to the View

| Privilege | Purpose |
|---|---|
| `comment_view` | Implicitly checked at start |
| `post_view` | Set as `canViewPosts` on the page context |

### View Event Listeners

| Event | Handler | Purpose |
|---|---|---|
| `submit` | `_evtUpdate` | Edit a comment's text |
| `score` | `_evtScore` | Set a comment's score |
| `delete` | `_evtDelete` | Delete a comment |

---

## Event Handlers

### `_evtUpdate(e)`
**Trigger:** User submits a comment edit.
**Parameters:** `e.detail.comment`, `e.detail.text`, `e.detail.target`

**Code path:**
1. Sets `e.detail.comment.text` to `e.detail.text`.
2. Calls `e.detail.comment.save()`.
3. On error: shows the error message on `e.detail.target`.

### `_evtScore(e)`
**Trigger:** User sets a score on a comment.
**Parameters:** `e.detail.comment`, `e.detail.score`

**Code path:**
1. Calls `e.detail.comment.setScore(e.detail.score)`.
2. On error: shows the error via `window.alert`.

### `_evtDelete(e)`
**Trigger:** User deletes a comment.
**Parameters:** `e.detail.comment`

**Code path:**
1. Calls `e.detail.comment.delete()`.
2. On error: shows the error via `window.alert`.

---

## API Calls

| Call | Purpose |
|---|---|
| `PostList.search("sort:comment-date comment-count:1..", offset, limit, fields)` | Fetches posts that have comments, sorted by comment date |
| `comment.save()` | Saves edited comment text |
| `comment.setScore(score)` | Sets the score on a comment |
| `comment.delete()` | Deletes a comment |

### Field Selection

The API request includes only these fields to minimize response size:
- `id`
- `comments`
- `commentCount`
- `thumbnailUrl`

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `PostList` | Searching for posts with comments |
| `top_navigation` | Navigation bar activation and title |
| `PageController` | Pagination logic |
| `CommentsPageView` | Renders the list of comments |
| `EmptyView` | Error view when missing privileges |
