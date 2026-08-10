# Models Layer — README

## Overview

The models layer defines all Diesel-backed data structures. Each file maps to one or more database tables. There are 11 doc files covering 22 database tables.

## Entity-Relationship Overview

```
user (1) ──< post (many)                  [uploader]
user (1) ──< comment (many)               [author]
user (1) ──< user_token (many)            [token owner]
user (1) ──< post_favorite (many)         [favorited by]
user (1) ──< post_score (many)            [scored by]
user (1) ──< post_feature (many)          [featured by]
user (1) ──< comment_score (many)         [scored by]
user (1) ──< snapshot (many)              [performed by]

post (1) ──< comment (many)               [comments on]
post (1) ──< post_note (many)             [annotations]
post (1) ──< post_favorite (many)         [favorites]
post (1) ──< post_score (many)            [scores]
post (1) ──< post_feature (many)          [features]
post (1) ──< post_tag (many)              [tag assignments]
post (1) ──< post_custom_field (many)     [custom field values]
post (1) ──< post_relation (many)         [parent relations]
post (1) ──  post_signature (1)           [perceptual signature]
post (1) ──  post_statistics (1)          [denormalized stats]

tag (1) ──< tag_name (many)               [aliases]
tag (1) ──< post_tag (many)               [post assignments]
tag (1) ──< tag_implication (many)        [implies others]
tag (1) ──< tag_suggestion (many)         [suggests others]
tag (1) ──< tag_implication (many)        [implied by others]
tag (1) ──  tag_statistics (1)            [denormalized stats]
tag (1) ──> tag_category (1)             [category]

pool (1) ──< pool_name (many)             [aliases]
pool (1) ──< pool_post (many)             [member posts]
pool (1) ──  pool_statistics (1)          [denormalized stats]
pool (1) ──> pool_category (1)           [category]

custom_field (1) ──< post_custom_field (many)  [assigned to posts]

tag_category (1) ──< tag (many)
pool_category (1) ──< pool (many)

comment (1) ──< comment_score (many)      [scores]
comment (1) ──  comment_statistics (1)    [denormalized stats]
```

## Database Tables (22)

### Core Entity Tables

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `post` | `id: Int8` | Main post entity with media metadata |
| `tag` | `id: Int8` | Tag entity with category and description |
| `pool` | `id: Int8` | Pool entity (ordered post collection) |
| `comment` | `id: Int8` | Comment entity (text on a post) |
| `user` | `id: Int8` | User account with credentials and rank |
| `custom_field` | `id: Int8` | Custom field definition |
| `tag_category` | `id: Int8` | Tag category (e.g., artist, character) |
| `pool_category` | `id: Int8` | Pool category |

### Name/Alias Tables

| Table | Composite PK | Description |
|-------|-------------|-------------|
| `tag_name` | `(tag_id, order)` | Tag name aliases (order 0 = primary) |
| `pool_name` | `(pool_id, order)` | Pool name aliases (order 0 = primary) |

### Junction / Relation Tables

| Table | Composite PK | Description |
|-------|-------------|-------------|
| `post_tag` | `(post_id, tag_id)` | Links posts to tags |
| `post_relation` | `(parent_id, child_id)` | Bidirectional post relationships |
| `post_favorite` | `(post_id, user_id)` | User favorites on posts |
| `post_score` | `(post_id, user_id)` | User scores (like/dislike) on posts |
| `post_custom_field` | `(post_id, custom_field_id)` | Custom field values on posts |
| `pool_post` | `(pool_id, post_id)` | Posts in pools with ordering |
| `tag_implication` | `(parent_id, child_id)` | Tag implication rules |
| `tag_suggestion` | `(parent_id, child_id)` | Tag suggestion rules |
| `comment_score` | `(comment_id, user_id)` | User scores on comments |

### Feature and Annotation Tables

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `post_feature` | `id: Int8` | Featured post history |
| `post_note` | `id: Int8` | Annotation notes with polygon coordinates |

### Audit and Token Tables

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `snapshot` | `id: Int8` | Audit trail of all resource changes |
| `user_token` | `id: Uuid` | API authentication tokens |
| `post_signature` | `post_id: Int8` | Perceptual signatures for duplicate detection |

### Denormalized Statistics Tables

| Table | Primary Key | Description |
|-------|-------------|-------------|
| `post_statistics` | `post_id` | Aggregated post stats (tag count, score, etc.) |
| `tag_statistics` | `tag_id` | Aggregated tag stats (usage, implication counts) |
| `pool_statistics` | `pool_id` | Aggregated pool stats (post count) |
| `comment_statistics` | `comment_id` | Aggregated comment stats (score) |
| `user_statistics` | `user_id` | Aggregated user stats (uploads, comments, favorites) |
| `tag_category_statistics` | `category_id` | Aggregated category usage count |
| `pool_category_statistics` | `category_id` | Aggregated category usage count |
| `database_statistics` | `id: Bool` (singleton) | Global stats (post count, disk usage, signature version) |

## Design Patterns

### New* Insert Structs

Entities that require only a subset of fields on creation use a separate `New*` struct with borrowed fields (`&'a str`). These implement `Insertable`:

| Model | New Struct | Notes |
|-------|-----------|-------|
| `Post` | `NewPost` | Includes all non-generated fields |
| `Tag` | `NewTag` | Only `category_id` and `description` |
| `Pool` | `NewPool` | `category_id`, `description`, `launch_date` |
| `Comment` | `NewComment` | `user_id`, `post_id`, `text`, `creation_time` |
| `User` | `NewUser` | `name`, password hash/salt, `email`, `rank`, `avatar_style` |
| `CustomField` | `NewCustomField` | `name`, `description`, `color` |
| `TagCategory` | `NewTagCategory` | `order`, `name`, `color` |
| `PoolCategory` | `NewPoolCategory` | `name`, `color` |
| `Snapshot` | `NewSnapshot` | All fields except `id` and `creation_time` |
| `PostFeature` | `NewPostFeature` | `post_id`, `user_id`, `time` |
| `UserToken` | `NewUserToken` | `id` (UUID), `user_id`, `note`, `enabled`, `expiration_time` |

### Direct Insertable (No New* Struct)

Junction tables and simple structs implement `Insertable` directly:

| Struct | Table |
|--------|-------|
| `PostTag` | `post_tag` |
| `PostRelation` | `post_relation` |
| `PostFavorite` | `post_favorite` |
| `PostScore` | `post_score` |
| `PostCustomField` | `post_custom_field` |
| `PoolPost` | `pool_post` |
| `TagImplication` | `tag_implication` |
| `TagSuggestion` | `tag_suggestion` |
| `CommentScore` | `comment_score` |

### Composite Keys

Eight tables use composite primary keys via `#[diesel(primary_key(field1, field2))]`:
`comment_score`, `pool_name`, `pool_post`, `post_custom_field`, `post_favorite`, `post_relation`, `post_score`, `post_tag`, `tag_implication`, `tag_name`, `tag_suggestion`.

### `AsChangeset`

Entities that support `save_changes()` derive `AsChangeset`: `CustomField`, `Pool`, `PoolCategory`, `Post`, `Tag`, `TagCategory`, `UserToken`. This allows the entity struct itself to act as an update payload.

### `treat_none_as_null`

`Post` and `UserToken` use `#[diesel(treat_none_as_null = true)]`. This tells Diesel to treat Rust `Option::None` as SQL `NULL` rather than skipping the column in UPDATE statements.

### Primary Name Optimization

`TagName::primary()` and `PoolName::primary()` use raw SQL literals (`order = 0`) instead of `order.eq(0)`. This gives up to 15x query speed improvement because PostgreSQL optimizes queries better when the exact value is known at planning time.

### Default Category Pattern

`TagCategory::default()` and `PoolCategory::default()` filter by `id = 0`. The category with ID 0 is the default category. Setting a new default swaps IDs with the current default via `std::mem::swap`.

## Enum Types and Integer Representations

All database-backed enums use `#[repr(i16)]` and serialize as `SmallInt`:

| Enum | Variants | Repr Range |
|------|----------|------------|
| `UserRank` | Anonymous, Restricted, Regular, Power, Moderator, Administrator | 0–5 |
| `PostSafety` | Safe, Sketchy, Unsafe | 0–2 |
| `PostType` | Image, Animation, Video, Document | 0–3 |
| `MimeType` | Bmp, Gif, Jpeg, Png, Webp, Mp4, Mov, Webm, Avif, Pdf | 0–9 |
| `AvatarStyle` | Gravatar, Manual | 0–1 |
| `ResourceOperation` | Created, Modified, Merged, Deleted | 0–3 |
| `ResourceType` | Comment, CustomField, Pool, PoolCategory, Post, Tag, TagCategory, TagImplication, TagSuggestion, User, UserToken | 0–10 |
| `Score` | Dislike = -1, Like = 1 | -1, 1 |
| `Rating` | Dislike = -1, None = 0, Like = 1 | -1, 0, 1 |
| `PostFlag` | Loop = 0, Sound = 1 | Bit positions |

### Non-database types
- `Rating` is API-only (not stored in DB). Converts to/from `Score` via `TryFrom`.
- `PostFlags` is a `u16` bitmask wrapper. Multiple `PostFlag` values are combined via bitwise OR.
- `ResourceProperty` describes specific properties of resources (used in snapshot/error context).

## Performance Optimizations

1. **Denormalized statistics tables** avoid COUNT(*) queries on junction tables. Each write to a junction table also updates the corresponding statistics row.
2. **Primary name SQL literals** (`order = 0`) avoid query planning overhead.
3. **`DISTINCT` for multivalued filters** — The `apply_distinct_if_multivalued!` macro adds DISTINCT only when needed (comma, wildcard, or range in the condition).
4. **SSO strings** — `SmallString` avoids allocation for short strings (≤24 bytes).
5. **`Arc<str>` for large strings** — `LargeString` shares immutable strings without copying.
6. **Content caching** — `AppState.content_cache` is a ring buffer (10 slots) for uploaded content.
7. **TEMP table filter cache** — Complex search filters use session-local TEMP tables to avoid stacking multiple independent JOINs.

## Documentation Files

| File | Content |
|------|---------|
| [enums.md](enums.md) | All enum types (`UserRank`, `PostSafety`, `MimeType`, `Score`, `ResourceOperation`, etc.), error types, `PostFlags` bitmask |
| [post.md](post.md) | `Post`, `NewPost`, `PostRelation`, `PostTag`, `PostFavorite`, `PostFeature`, `PostNote`, `PostScore`, `PostSignature`, `NewPostSignature`, `CompressedSignature`, `SignatureIndexes` |
| [tag.md](tag.md) | `Tag`, `NewTag`, `TagName`, `NewTagName`, `TagImplication`, `TagSuggestion` |
| [pool.md](pool.md) | `Pool`, `NewPool`, `PoolName`, `NewPoolName`, `PoolPost` |
| [comment.md](comment.md) | `Comment`, `NewComment`, `CommentScore`, `NewCommentScore` |
| [user.md](user.md) | `User`, `NewUser`, `UserToken`, `NewUserToken` |
| [custom_field.md](custom_field.md) | `CustomField`, `NewCustomField` |
| [post_custom_field.md](post_custom_field.md) | `PostCustomField` junction table |
| [snapshot.md](snapshot.md) | `Snapshot`, `NewSnapshot` |
| [tag_category.md](tag_category.md) | `TagCategory`, `NewTagCategory` |
| [pool_category.md](pool_category.md) | `PoolCategory`, `NewPoolCategory` |
