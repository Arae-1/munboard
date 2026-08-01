# Snapshots Controller

**File:** `client/js/controllers/snapshots_controller.js`
**Class:** `SnapshotsController`

## Purpose

Controls the history (snapshots) page. Shows a paginated list of recent changes across all entities (posts, tags, users, etc.).

## Routing

| Route | Controller | Handler |
|---|---|---|
| `history` | `SnapshotsController` | Constructor |

---

## Constructor

**Parameters:** `ctx` (the router context with URL parameters)

**Code path:**
1. Checks if the user has the `snapshot_list` privilege.
2. If the user does **not** have the privilege:
   - Creates an `EmptyView`.
   - Shows "You don't have privileges to view history.".
   - Returns early.
3. Activates an empty string `""` in the top navigation bar (no item highlighted).
4. Sets the page title to "History".
5. Creates a `PageController` instance.
6. Runs the page controller with the following configuration.

### Page Controller Configuration

| Option | Value |
|---|---|
| `parameters` | `ctx.parameters` |
| `defaultLimit` | `25` |
| `getClientUrlForPage` | Returns client URL for "history" with offset and limit |
| `requestPage` | Calls `SnapshotList.search("", offset, limit)` with empty query |
| `pageRenderer` | Creates a `SnapshotsPageView` with privileges |

### Privileges Passed to the View (via pageCtx)

| Privilege | Context Property |
|---|---|
| `post_view` | `canViewPosts` |
| `user_view` | `canViewUsers` |
| `tag_view` | `canViewTags` |

---

## Notes

- No event listeners. The page is read-only.
- No `showSuccess` or `showError` methods. The controller does not expose these to external callers.
- The search query is hardcoded to `""` (empty). There is no search form on this page.
- The `requestPage` function ignores the extra `fields` or `options` parameters.

---

## API Calls

| Call | Purpose |
|---|---|
| `SnapshotList.search("", offset, limit)` | Fetches the latest snapshots |

---

## Dependencies

| Module | Use |
|---|---|
| `api` | Privilege checks |
| `uri` | Client link formatting |
| `SnapshotList` | Fetches snapshots |
| `PageController` | Pagination logic |
| `top_navigation` | Navigation bar activation and title |
| `SnapshotsPageView` | Renders the history list |
| `EmptyView` | Error view when missing privileges |
