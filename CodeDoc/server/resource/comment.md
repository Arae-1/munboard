# Comment Resource — `resource/comment.rs`

**Source:** `server/src/resource/comment.rs` (167 lines)

## Purpose

This file serializes `Comment` model entities into `CommentInfo` API response structs. Comments belong to posts and contain Markdown text. The resource supports field selection and batch loading of user, score, and own-score data.

## Dependencies

- `crate::app::Context` — application context (config, client)
- `crate::auth::Client` — authenticated user identity
- `crate::model::comment::{Comment, CommentScore}` — database models
- `crate::model::enums::{AvatarStyle, Rating}` — enum types
- `crate::resource` — shared helpers (`single`, `order_as`, `order_like`)
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::resource::user::MicroUser` — lightweight user representation
- `crate::schema::{comment, comment_score, comment_statistics, user}` — Diesel schema
- `crate::string::{LargeString, SmallString}` — string types
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM query building
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Field Enum

```rust
pub enum Field {
    Version,
    Id,
    PostId,
    Text,
    CreationTime,
    LastEditTime,
    User,
    Score,
    OwnScore,
}
```

Each variant serializes as camelCase in query strings (e.g. `fields=id,text,user`). The `From<Field> for u64` impl converts each variant to its discriminant value for bitmask operations.

## Struct

### `CommentInfo`

The API response shape for a comment.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `comment.last_edit_time` | Resource version for cache validation |
| `id` | `Option<i64>` | `comment.id` | Comment identifier |
| `post_id` | `Option<i64>` | `comment.post_id` | Parent post ID |
| `text` | `Option<LargeString>` | `comment.text` | Markdown content |
| `creation_time` | `Option<DateTime>` | `comment.creation_time` | When created |
| `last_edit_time` | `Option<DateTime>` | `comment.last_edit_time` | When last edited |
| `user` | `Option<Option<MicroUser>>` | Joined from `user` table | `null` = anonymous author. Inner `None` = field not requested. |
| `score` | `Option<i64>` | `comment_statistics.score` | Collective score |
| `own_score` | `Option<Rating>` | `comment_score.score` filtered by client | Authenticated user's vote, or `Rating::default()` if none |

**Serialization:** All fields use `skip_serializing_none`. The `non_nullable_options` macro ensures `Option` fields are treated as `#[serde(skip_serializing_if = "Option::is_none")]`.

## Constructor Methods

### `CommentInfo::new(conn, ctx, comment, fields) -> QueryResult<Self>`

Create a single comment resource from a model entity. Delegates to `new_batch` and calls `resource::single` to unwrap.

### `CommentInfo::new_from_id(conn, ctx, comment_id, fields) -> QueryResult<Self>`

Create a single comment resource by ID. Loads the comment from the database, then delegates to `new_batch`.

### `CommentInfo::new_batch(conn, ctx, comments, fields) -> QueryResult<Vec<Self>>`

Create multiple comment resources in one batch.

**Processing order:**
1. Create `Batcher` with field mask and comment count
2. Conditionally load owners via `get_owners`
3. Conditionally load scores via `get_scores`
4. Conditionally load own-scores via `get_own_scores`
5. Iterate over comments in reverse, popping pre-loaded data
6. Reverse result to restore original order

**Why reverse-and-pop:** The `Batcher` returns vectors in forward order. `.pop()` removes from the end. By iterating in reverse, the first comment matches the first pre-loaded value.

### `CommentInfo::new_batch_from_ids(conn, ctx, comment_ids, fields) -> QueryResult<Vec<Self>>`

Load comments by IDs. Applies `resource::order_as` to reorder database results to match input ID order. Delegates to `new_batch`.

## Helper Functions

### `get_owners(conn, config, comments) -> QueryResult<Vec<Option<MicroUser>>>`

Load comment authors. Inner-joins `comment` with `user` to get `(comment_id, name, avatar_style)`. Uses `resource::order_like` to map results to input order, padding missing owners (anonymous comments) with `None`.

**Database query:**
```sql
SELECT comment.id, user.name, user.avatar_style
FROM comment
INNER JOIN user ON comment.user_id = user.id
WHERE comment.id IN (...)
```

**Transform:** Each `(_, username, avatar_style)` tuple becomes a `MicroUser::new(config, username, avatar_style)`.

### `get_scores(conn, comments) -> QueryResult<Vec<i64>>`

Load comment scores from `comment_statistics`. Returns scores ordered to match input comments.

**Database query:**
```sql
SELECT comment_statistics.comment_id, comment_statistics.score
FROM comment_statistics
WHERE comment_statistics.comment_id IN (...)
```

**Transform:** Uses `resource::order_transformed_as` to reorder by comment ID, then maps to score values.

### `get_own_scores(conn, client, comments) -> QueryResult<Vec<Rating>>`

Load the authenticated user's scores on the given comments. If the client is anonymous (`client.id` is `None`), returns `Vec<Rating::default()>` for all comments. Otherwise:

**Database query:**
```sql
SELECT * FROM comment_score
WHERE comment_score.comment_id IN (...)
  AND comment_score.user_id = <client_id>
```

**Transform:** Uses `resource::order_like` to map results. Converts `Score` to `Rating` via `Rating::from(score.score)`. Missing entries become `Rating::default()` (no vote).
