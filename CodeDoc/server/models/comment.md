# Comment Model

## Database Tables

- `comment` — Stores comments on posts.
- `comment_score` — Stores a score (like or dislike) that a user gave to a comment.

## Structs

### `NewComment` (Insertable)

Maps to table: `comment`

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Option<i64>` | ID of the user who wrote the comment. Null if the user is anonymous. |
| `post_id` | `i64` | ID of the post the comment belongs to. |
| `text` | `&'a str` | The comment text content. |
| `creation_time` | `DateTime` | The time the comment was created. |

Diesel annotations:
- `#[diesel(table_name = comment)]`
- `#[diesel(check_for_backend(Pg))]`

### `Comment`

Maps to table: `comment`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `user_id` | `Option<i64>` | Foreign key to `user.id`. Null for anonymous. |
| `post_id` | `i64` | Foreign key to `post.id`. |
| `text` | `LargeString` | The comment text content. |
| `creation_time` | `DateTime` | Timestamp when the comment was created. |
| `last_edit_time` | `DateTime` | Timestamp when the comment was last edited. |

Diesel annotations:
- `#[diesel(table_name = comment)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(User), belongs_to(Post))]`

Relationships:
- Belongs to `User` via `user_id`.
- Belongs to `Post` via `post_id`.

### `NewCommentScore` (Insertable)

Maps to table: `comment_score`

| Field | Type | Description |
|-------|------|-------------|
| `comment_id` | `i64` | ID of the comment being scored. |
| `user_id` | `i64` | ID of the user who scored the comment. |
| `score` | `Score` | The score value. `Score::Like` (1) or `Score::Dislike` (-1). |

Diesel annotations:
- `#[diesel(table_name = comment_score)]`
- `#[diesel(check_for_backend(Pg))]`

### `CommentScore`

Maps to table: `comment_score`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `comment_id` | `i64` | Part of composite primary key. Foreign key to `comment.id`. |
| `user_id` | `i64` | Part of composite primary key. Foreign key to `user.id`. |
| `score` | `Score` | The score value. |
| `time` | `DateTime` | Timestamp when the score was recorded. |

Diesel annotations:
- `#[diesel(table_name = comment_score)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(comment_id, user_id))]`
- `#[diesel(belongs_to(Comment), belongs_to(User))]`

Relationships:
- Belongs to `Comment` via `comment_id`.
- Belongs to `User` via `user_id`.

Composite primary key: `(comment_id, user_id)`.

## Type References

- `Score` — Defined in `enums.rs`. Stored as `SmallInt` (-1 or 1).
- `DateTime` — A timestamp type from `crate::time`.
- `LargeString` — A string type from `crate::string`.
