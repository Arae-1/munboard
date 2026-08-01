# Comment API — `comment.rs`

File: `server/src/api/comment.rs`

## Overview

This module defines all comment-related HTTP endpoints. It handles CRUD operations, search, and scoring.

---

## Router Registration

### `routes() -> OpenApiRouter<AppState>`

**Location:** `comment.rs:22`

Returns a router with these endpoints:

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/comments` | `list` |
| `GET` | `/comment/{id}` | `get` |
| `POST` | `/comments` | `create` |
| `PUT` | `/comment/{id}` | `update` |
| `PUT` | `/comment/{id}/score` | `rate` |
| `DELETE` | `/comment/{id}` | `delete` |

---

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_COMMENTS_PER_PAGE` | `1000` | Maximum comments per page in list queries |

---

## Helper: `verify_visibility`

**Location:** `comment.rs:340`

```rust
fn verify_visibility(conn, ctx, comment_id) -> ApiResult<()>
```

1. Checks if the comment exists in the database (`comment::table.find(comment_id)`).
2. If not found, returns `ApiError::NotFound(ResourceType::Comment)`.
3. Applies anonymous user preferences: checks if the comment belongs to a hidden post (via `preferences::hidden_posts`).
4. If hidden, returns `ApiError::Hidden(ResourceType::Comment)`.

---

## Endpoints

### 1. `GET /comments` — List Comments

**Handler:** `list` at `comment.rs:71`

**Authentication:** Requires `Action::CommentList`.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |
| `offset` | `Option<i64>` | Starting position (default: 0) |
| `limit` | `NonZeroI64` | Maximum results (capped at 1000) |

**Search Tokens (Named):**

| Token | Description |
|-------|-------------|
| `id` | Specific comment ID |
| `post` | Specific post ID |
| `user`, `author` | Author name (wildcards allowed) |
| `text` | Comment text (wildcards allowed) |
| `creation-date`, `creation-time` | Creation date |
| `last-edit-date`, `last-edit-time`, `edit-date`, `edit-time` | Last edit date |

**Sort Tokens:** `random`, `user`/`author`, `post`, `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time`

**Successful Response:** `200` with `PagedResponse<CommentInfo>`

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::CommentList` privilege.
2. Cap `limit` at `MAX_COMMENTS_PER_PAGE` (1000).
3. Open a database transaction.
4. Build `QueryBuilder` from the search query.
5. Set offset and limit.
6. Execute `query_builder.list(conn)` to get `(total, selected_comment_ids)`.
7. Batch-load `CommentInfo` from the selected comment IDs.
8. Return `PagedResponse` as JSON.

---

### 2. `GET /comment/{id}` — Get Single Comment

**Handler:** `get` at `comment.rs:113`

**Authentication:** Requires `Action::CommentView`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Comment ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `CommentInfo`

**Error Responses:**
- `403` — Privileges too low
- `403` — Comment is hidden (belongs to a blacklisted post for anonymous)
- `404` — Comment does not exist

**Handler Logic:**
1. Verify `Action::CommentView` privilege.
2. Open a database transaction.
3. Call `verify_visibility` to check existence and anonymity preferences.
4. Load `CommentInfo` from the comment ID with requested fields.
5. Return as JSON.

---

### 3. `POST /comments` — Create a Comment

**Handler:** `create` at `comment.rs:153`

**Authentication:** Requires `Action::CommentCreate`.

**Request Body:** `CommentCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `postId` | `i64` | ID of the post to comment on |
| `text` | `String` | Comment text |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `CommentInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Post does not exist (foreign key violation on `post_id`)

**Handler Logic:**
1. Verify `Action::CommentCreate` privilege.
2. Open a database transaction.
3. Insert `NewComment` into `comment::table` with the current time as `creation_time`.
4. Map foreign key violations to `ApiError::NotFound(ResourceType::Post)` (if the referenced post does not exist).
5. Record a `creation_snapshot` with the comment data.
6. After the first transaction, load `CommentInfo` and return as JSON.

---

### 4. `PUT /comment/{id}` — Update a Comment

**Handler:** `update` at `comment.rs:209`

**Authentication:** If the client owns the comment, requires rank >= `config.privileges()[Action::CommentEditOwn]`. Otherwise, requires rank >= `config.privileges()[Action::CommentEditAny]`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Comment ID |

**Request Body:** `CommentUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version (required) |
| `text` | `String` | New comment text |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `CommentInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Comment does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Look up the required ranks for `CommentEditOwn` and `CommentEditAny` from config.
2. Open a database transaction.
3. Load the existing comment. Return `NotFound` if absent.
4. Verify version.
5. Determine ownership: the client owns the comment if `client.id == comment.user_id && comment.user_id.is_some()`.
6. Select the required rank (`edit_own` or `edit_any`) and call `api::verify_privilege(client, required_rank)`.
7. Update the comment text and set `last_edit_time` to now.
8. Record a `modification_snapshot` with old and new comment data.
9. After the first transaction, load `CommentInfo` and return as JSON.

---

### 5. `PUT /comment/{id}/score` — Rate a Comment

**Handler:** `rate` at `comment.rs:264`

**Authentication:** Requires `Action::CommentScore`. User must be logged in.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Comment ID |

**Request Body:** `RatingBody`

| Field | Type | Description |
|-------|------|-------------|
| `score` | `Rating` | Score value (-1, 0, 1) |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `CommentInfo`

**Error Responses:**
- `400` — Score is invalid
- `403` — Privileges too low
- `404` — Comment does not exist

**Handler Logic:**
1. Verify `Action::CommentScore` privilege.
2. Extract user ID (return `NotLoggedIn` if anonymous).
3. Open a database transaction.
4. Delete any existing score for this user+comment pair.
5. If the score is a valid `Score` value (-1 or 1, not 0), insert a `NewCommentScore` record. Score of 0 removes the rating (no insert).
6. Map foreign key violations to `ResourceType::Comment`.
7. After the first transaction, load `CommentInfo` and return as JSON.

---

### 6. `DELETE /comment/{id}` — Delete a Comment

**Handler:** `delete` at `comment.rs:312`

**Authentication:** If the client owns the comment, requires `Action::CommentDeleteOwn`. Otherwise, requires `Action::CommentDeleteAny`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Comment ID |

**Request Body:** `DeleteBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |

**Successful Response:** `200` with `()`

**Error Responses:**
- `403` — Privileges too low
- `404` — Comment does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Open a database transaction.
2. Load the comment. Return `NotFound` if absent.
3. Verify version.
4. Determine ownership: the client owns the comment if `client.id == comment.user_id && comment.user_id.is_some()`.
5. Select action: `CommentDeleteOwn` or `CommentDeleteAny`. Verify privilege.
6. Delete the comment from `comment::table`.
7. Record a `deletion_snapshot` with the comment data.
8. Return `Json(())`.

---

## Type Definitions

### `CommentCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `postId` | `i64` | Post ID to comment on |
| `text` | `String` | Comment text |

### `CommentUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |
| `text` | `String` | New comment text |
