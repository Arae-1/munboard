# User Resource — `resource/user.rs`

**Source:** `server/src/resource/user.rs` (235 lines)

## Purpose

This file serializes `User` model entities into `UserInfo` API response structs. Users have public and private data fields. The `Visibility` enum controls which fields are exposed. This file also defines `MicroUser`, a lightweight user representation, and `PrivateData`, an enum that hides or exposes private fields.

## Dependencies

- `crate::config::Config` — application configuration
- `crate::content::hash` — gravatar URL generation
- `crate::model::enums::{AvatarStyle, Score, UserRank}` — enum types
- `crate::model::post::PostScore` — post score model for liked/disliked counts
- `crate::model::user::User` — database model
- `crate::resource` — shared helpers (`single`, `order_as`, `order_transformed_as`, `order_like`)
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::schema::{post_score, user}` — Diesel schema
- `crate::string::SmallString` — compact string type
- `crate::time::DateTime` — timestamp type
- `crate::user_stats!` — macro for user statistics queries
- `diesel` — ORM including `BelongingToDsl`, `GroupedBy`
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Structs and Enums

### `MicroUser`

A lightweight user representation. Used by every resource that references a user (post uploader, comment author, pool poster, snapshot creator, favorited-by lists).

| Field | Type | Source | Purpose |
|-------|------|--------|---------|
| `name` | `SmallString` | `user.name` | User name |
| `avatar_url` | `String` | Computed | URL to the user's avatar image |

**Constructor:**

```rust
MicroUser::new(config, name: SmallString, avatar_style: AvatarStyle) -> Self
```

**Avatar URL computation:**
- If `avatar_style` is `AvatarStyle::Gravatar`: generates a Gravatar URL via `hash::gravatar_url(config, &name)`. The URL includes the email hash and configurable Gravatar parameters.
- If `avatar_style` is `AvatarStyle::Manual`: generates a custom avatar URL via `config.custom_avatar_url(&name)`.

### `Visibility`

Controls which user fields are visible in the API response.

| Variant | Meaning |
|---------|---------|
| `Full` | All fields visible. Used when the authenticated user views their own profile, or an admin views any profile. |
| `PublicOnly` | Private fields are hidden. Used for public profile views. |

### `PrivateData<T>`

An enum that controls visibility of private user data.

| Variant | Serialized As | Usage |
|---------|--------------|-------|
| `Expose(T)` | The actual value of `T` | Used when visibility is `Full` |
| `Visible(bool)` | `false` | Used when visibility is `PublicOnly` |

**Serialization:** `#[serde(untagged)]` — the variant is selected based on the data shape, not a type tag. Consumers see either the real value or `false`.

### `UserInfo`

The full API response shape for a user.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `user.last_edit_time` | Resource version |
| `name` | `Option<SmallString>` | `user.name` | User name |
| `email` | `Option<PrivateData<Option<SmallString>>>` | `user.email` | Hidden behind `PrivateData`. Exposed for Full visibility, `false` for PublicOnly. `null` if user has no email. |
| `rank` | `Option<UserRank>` | `user.rank` | Privilege level |
| `last_login_time` | `Option<DateTime>` | `user.last_login_time` | Last login timestamp |
| `creation_time` | `Option<DateTime>` | `user.creation_time` | Registration timestamp |
| `avatar_style` | `Option<AvatarStyle>` | `user.avatar_style` | Avatar rendering method |
| `avatar_url` | `Option<String>` | Computed via `user.avatar_url(config)` | The avatar URL |
| `comment_count` | `Option<i64>` | `user_statistics.comment_count` | Number of comments |
| `uploaded_post_count` | `Option<i64>` | `user_statistics.upload_count` | Number of uploaded posts |
| `liked_post_count` | `Option<PrivateData<i64>>` | `post_score` count with `Score::Like` | Hidden behind `PrivateData` |
| `disliked_post_count` | `Option<PrivateData<i64>>` | `post_score` count with `Score::Dislike` | Hidden behind `PrivateData` |
| `favorite_post_count` | `Option<i64>` | `user_statistics.favorite_count` | Number of favorited posts |

**Private fields:** `email`, `liked_post_count`, and `disliked_post_count` use `PrivateData<T>`. They show the real value only when visibility is `Full`.

## Field Enum

```rust
pub enum Field {
    Version, Name, Email, Rank, LastLoginTime, CreationTime,
    AvatarStyle, AvatarUrl,
    CommentCount, UploadedPostCount,
    LikedPostCount, DislikedPostCount,
    FavoritePostCount,
}
```

## Constructor Methods

### `UserInfo::new(conn, config, user, fields, visibility) -> QueryResult<Self>`

Create a single user resource. Delegates to `new_batch` and calls `resource::single`. Accepts a `visibility` parameter to control private data.

### `UserInfo::new_from_id(conn, config, user_id, fields, visibility) -> QueryResult<Self>`

Load from ID. Uses `resource::order_as`. Delegates to `new_batch`.

### `UserInfo::new_batch(conn, config, users, fields, visibility) -> QueryResult<Vec<Self>>`

Create multiple user resources. Uses `Batcher` for conditional loading.

**Conditionally loaded data:**

| Field | Helper Function | Data Source |
|-------|----------------|-------------|
| `CommentCount` | `user_stats!` macro | `user_statistics.comment_count` |
| `UploadedPostCount` | `user_stats!` macro | `user_statistics.upload_count` |
| `LikedPostCount` | `get_ratings(conn, &users, Score::Like, visibility)` | `post_score` grouped by user, filtered by score |
| `DislikedPostCount` | `get_ratings(conn, &users, Score::Dislike, visibility)` | `post_score` grouped by user, filtered by score |
| `FavoritePostCount` | `user_stats!` macro | `user_statistics.favorite_count` |

**Visibility in batch construction:**
- `email`: `PrivateData::Expose(user.email)` for Full, `PrivateData::Visible(false)` for PublicOnly
- `liked_post_count` / `disliked_post_count`: The `get_ratings` function handles visibility internally

### `UserInfo::new_batch_from_ids(conn, config, user_ids, fields, visibility) -> QueryResult<Vec<Self>>`

Load users by IDs. Uses `resource::order_as`. Delegates to `new_batch`.

## Helper Functions

### `get_ratings(conn, users, score, visibility) -> QueryResult<Vec<PrivateData<i64>>>`

Load liked or disliked post counts for a batch of users.

**Early return for PublicOnly:**
```rust
if visibility == Visibility::PublicOnly {
    return Ok(vec![PrivateData::Visible(false); users.len()]);
}
```

**Database query (Full visibility only):**
```sql
SELECT post_score.user_id, COUNT(*)
FROM post_score
WHERE post_score.user_id IN (...)
  AND post_score.score = <Like or Dislike>
GROUP BY post_score.user_id
```

**Transform:** Uses `resource::order_like` to align results. Missing entries (users with no likes/dislikes) map to 0. All results are wrapped in `PrivateData::Expose`.

## Macros

### `user_stats!`

A declarative macro that queries `user_statistics` for a batch of users.

**Signature:**
```rust
user_stats!($conn, $users, $column)
```

**Generated code:** Same pattern as `post_stats!`:
1. Collect user IDs
2. Query `user_statistics` selecting `(user_id, column)`
3. Use `resource::order_transformed_as` to reorder
4. Map to column values as `Vec<i64>`

**Usage:**
```rust
let mut comment_counts = f.exec(Field::CommentCount, || user_stats!(conn, &users, comment_count))?;
```
