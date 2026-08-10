# Post Resource — `resource/post.rs`

**Source:** `server/src/resource/post.rs` (634 lines)

## Purpose

This file serializes `Post` model entities into `PostInfo` API response structs. Posts are the central entity of the application — each represents one uploaded file with metadata, tags, comments, relations, pools, notes, scores, and statistics. This is the largest resource file in the module.

## Dependencies

- `crate::app::Context` — application context (config, client)
- `crate::auth::Client` — authenticated user identity
- `crate::config::Config` — application configuration
- `crate::content::hash::PostHash` — content/thumbnail URL generation and custom thumbnail checks
- `crate::model::comment::Comment` — comment model
- `crate::model::enums::{AvatarStyle, MimeType, PostFlags, PostSafety, PostType, Rating, Score}` — enum types
- `crate::model::pool::PoolPost` — pool-post association model
- `crate::model::post::{NewPostNote, Post, PostFavorite, PostNote, PostRelation, PostScore, PostTag}` — post-related models
- `crate::model::tag::TagName` — tag name model
- `crate::post_stats!` — macro for post statistics queries
- `crate::resource` — shared helpers (`single`, `order_as`, `order_like`, `order_transformed_as`, `collect_names`)
- `crate::resource::comment::CommentInfo` — comment resource type
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::resource::pool::MicroPool` — lightweight pool
- `crate::resource::tag::MicroTag` — lightweight tag
- `crate::resource::user::MicroUser` — lightweight user
- `crate::resource::custom_field::CustomFieldValue` — custom field values
- `crate::schema` — Diesel schema tables for post, comment, pool, tag, user, and related join tables
- `crate::search::preferences` — user preference filtering
- `crate::string::{LargeString, SmallString}` — string types
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM including `GroupedBy`, `BelongingToDsl`, `NullableExpressionMethods`
- `serde::{Deserialize, Serialize}` — serialization and deserialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `std::collections::{HashMap, HashSet}` — data structures
- `std::convert::Infallible` — for infallible Result types
- `std::sync::Arc` — shared ownership
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Structs

### `Note`

A polygon annotation on a post. Notes allow users to draw boxes on images and add text.

| Field | Type | Source | Serialized? | Purpose |
|-------|------|--------|-------------|---------|
| `id` | `i64` | `post_note.id` | No (`#[serde(skip)]`) | Internal database ID |
| `polygon` | `Vec<[f32; 2]>` | `post_note.polygon` | Yes | Vertex coordinates in [0,1] range |
| `text` | `LargeString` | `post_note.text` | Yes | Markdown annotation text |

**Serialization:** `id` is skipped. Only `polygon` and `text` appear in API responses.

**Methods:**

#### `Note::new(note: PostNote) -> Self`

Convert a `PostNote` model to a `Note` resource. The model stores polygon points as a flat `Vec<Option<f32>>` array. The resource converts it to `Vec<[f32; 2]>` by chunking into pairs of 2. Each vertex is unwrapped from `Option` — panics on `None` because the database should not contain NULL vertices.

**Example:** Flat array `[Some(0.0), Some(0.0), Some(1.0), Some(0.0)]` becomes `[[0.0, 0.0], [1.0, 0.0]]`.

#### `Note::to_new_post_note(&self, post_id: i64) -> NewPostNote`

Reverse conversion. Converts the resource `Note` back to a `NewPostNote` for database insertion. Flattens the polygon `Vec<[f32; 2]>` back to a `Vec<f32>`.

#### `Note::id(&self) -> i64`

Returns the internal database ID. Used when updating existing notes.

### `MicroPost`

A lightweight post representation. Used by pool posts, relations, and anywhere a full post is not needed.

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `i64` | Post identifier |
| `thumbnail_url` | `String` | URL to the post thumbnail |

**Serialization:** Both fields are always serialized.

### `PostInfo`

The full API response shape for a post. This is the largest resource struct with 30 optional fields.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `post.last_edit_time` | Resource version |
| `id` | `Option<i64>` | `post.id` | Post identifier |
| `user` | `Option<Option<MicroUser>>` | Joined from `user` table | Post uploader. `None` outer = field not requested |
| `file_size` | `Option<i64>` | `post.file_size` | File size in bytes |
| `canvas_width` | `Option<i32>` | `post.width` | Image width in pixels |
| `canvas_height` | `Option<i32>` | `post.height` | Image height in pixels |
| `safety` | `Option<PostSafety>` | `post.safety` | Safety rating (safe/sketchy/unsafe) |
| `type_` | `Option<PostType>` | `post.type_` | Content type (image/video/animation/document) |
| `mime_type` | `Option<MimeType>` | `post.mime_type` | Exact MIME format |
| `checksum` | `Option<String>` | `post.checksum` | BLAKE3 hash, hex-encoded via `hex::encode` |
| `checksum_md5` | `Option<String>` | `post.checksum_md5` | MD5 hash, hex-encoded via `hex::encode` |
| `flags` | `Option<PostFlags>` | `post.flags` | Bit flags (e.g. loop animation) |
| `source` | `Option<LargeString>` | `post.source` | Original source URL |
| `description` | `Option<LargeString>` | `post.description` | Markdown description |
| `launch_date` | `Option<Option<DateTime>>` | Computed | Earliest pool launch date, or post.launch_date |
| `media_created_date` | `Option<Option<DateTime>>` | `post.media_created_date` | Media creation date |
| `creation_time` | `Option<DateTime>` | `post.creation_time` | When uploaded |
| `last_edit_time` | `Option<DateTime>` | `post.last_edit_time` | When last edited |
| `content_url` | `Option<String>` | Computed via `PostHash` | URL to the file content |
| `thumbnail_url` | `Option<String>` | Computed via `PostHash` | URL to the thumbnail |
| `tags` | `Option<Vec<MicroTag>>` | Joined from tag tables | Tags on the post |
| `comments` | `Option<Vec<CommentInfo>>` | From comment tables | Comments on the post |
| `relations` | `Option<Vec<MicroPost>>` | From `post_relation` | Related posts |
| `pools` | `Option<Vec<MicroPool>>` | From `pool_post` | Pools containing this post |
| `notes` | `Option<Vec<Note>>` | From `post_note` | Polygon annotations |
| `score` | `Option<i64>` | `post_statistics.score` | Collective score |
| `own_score` | `Option<Rating>` | From `post_score` filtered by client | Authenticated user's vote |
| `own_favorite` | `Option<bool>` | From `post_favorite` filtered by client | Whether authenticated user favorited |
| `tag_count` | `Option<i64>` | `post_statistics.tag_count` | Number of tags |
| `comment_count` | `Option<i64>` | `post_statistics.comment_count` | Number of comments |
| `relation_count` | `Option<i64>` | `post_statistics.relation_count` | Number of related posts |
| `note_count` | `Option<i64>` | `post_statistics.note_count` | Number of notes |
| `favorite_count` | `Option<i64>` | `post_statistics.favorite_count` | Number of users who favorited |
| `feature_count` | `Option<i64>` | `post_statistics.feature_count` | Times featured |
| `last_feature_time` | `Option<Option<DateTime>>` | `post_statistics.last_feature_time` | Last time featured |
| `favorited_by` | `Option<Vec<MicroUser>>` | From `post_favorite` joined with `user` | Users who favorited |
| `has_custom_thumbnail` | `Option<bool>` | Filesystem check | Whether custom thumbnail file exists |
| `custom_fields` | `Option<Vec<CustomFieldValue>>` | From `post_custom_field` | Custom field values |

### Special Transformations

**`checksum` / `checksum_md5`:** Both are stored as `Vec<u8>` in the model. They are hex-encoded via `hex::encode()` for the API response.

**`launch_date`:** Uses a computed value. First checks `get_pool_launch_dates` for the earliest pool launch date containing this post. If no pool launch date exists, falls back to `post.launch_date`. The expression is:
```rust
pool_launch_dates.pop().unwrap_or(None).or(post.launch_date)
```

**`has_custom_thumbnail`:** Checks the filesystem instead of the database. Calls `PostHash::new(&ctx.config, post.id).custom_thumbnail_path().exists()`. This is an I/O operation, not a database query.

## Field Enum

```rust
pub enum Field {
    Version, Id, User, FileSize, CanvasWidth, CanvasHeight,
    Safety, Type, MimeType, Checksum, ChecksumMd5, Flags,
    Source, Description, LaunchDate, MediaCreatedDate,
    CreationTime, LastEditTime, ContentUrl, ThumbnailUrl,
    Tags, Comments, Relations, Pools, Notes,
    Score, OwnScore, OwnFavorite,
    TagCount, CommentCount, RelationCount, NoteCount,
    FavoriteCount, FeatureCount, LastFeatureTime,
    FavoritedBy, HasCustomThumbnail, CustomFields,
}
```

This is the largest Field enum with 30 variants. Each converts to its discriminant (0-29) for bitmask operations.

## Constructor Methods

### `PostInfo::new(conn, ctx, post, fields) -> QueryResult<Self>`

Create a single post resource. Delegates to `new_batch` and calls `resource::single`.

### `PostInfo::new_from_id(conn, ctx, post_id, fields) -> QueryResult<Self>`

Load from ID, then delegate to `new_batch`.

### `PostInfo::new_batch(conn, ctx, posts, fields) -> QueryResult<Vec<Self>>`

The main batch constructor. Creates a `Batcher` and conditionally loads all related data.

**Conditionally loaded data (20 separate batches):**

| Field | Helper Function | Data Source |
|-------|----------------|-------------|
| `User` | `get_owners` | `post` INNER JOIN `user` |
| `ContentUrl` | `get_content_urls` | Computed via `PostHash` (no DB query, returns `Result<_, Infallible>`) |
| `ThumbnailUrl` | `get_thumbnail_urls` | Computed via `PostHash` (no DB query) |
| `Tags` | `get_tags` | Multi-table join: post_tag, tag, tag_statistics, tag_category, tag_name |
| `Comments` | `get_comments` | Multi-table join: comment, comment_statistics, user, comment_score |
| `Relations` | `get_relations` | `post_relation` filtered by preferences |
| `Pools` | `get_pools` | Multi-table join: pool_post, pool, pool_statistics, pool_name, pool_category |
| `LaunchDate` | `get_pool_launch_dates` | `pool_post` INNER JOIN `pool`, filtered by non-null launch_date |
| `Notes` | `get_notes` | `PostNote::belonging_to(posts)` |
| `Score` | `post_stats!` macro | `post_statistics.score` |
| `OwnScore` | `get_own_scores` | `post_score` filtered by client user ID |
| `OwnFavorite` | `get_own_favorites` | `post_favorite` filtered by client user ID |
| `TagCount` | `post_stats!` macro | `post_statistics.tag_count` |
| `CommentCount` | `post_stats!` macro | `post_statistics.comment_count` |
| `RelationCount` | `post_stats!` macro | `post_statistics.relation_count` |
| `NoteCount` | `post_stats!` macro | `post_statistics.note_count` |
| `FavoriteCount` | `post_stats!` macro | `post_statistics.favorite_count` |
| `FeatureCount` | `post_stats!` macro | `post_statistics.feature_count` |
| `LastFeatureTime` | `post_stats!` macro | `post_statistics.last_feature_time` |
| `FavoritedBy` | `get_favorited_by` | `post_favorite` INNER JOIN `user` |
| `CustomFields` | `custom_field::get_custom_fields_for_posts` | `post_custom_field` INNER JOIN `custom_field` |

**ContentUrl and ThumbnailUrl are infallible:** `get_content_urls` and `get_thumbnail_urls` return `Result<_, Infallible>`. The code uses `let Ok(mut ...) = f.exec(...)` to destructure because the result can never actually fail.

### `PostInfo::new_batch_from_ids(conn, ctx, post_ids, fields) -> QueryResult<Vec<Self>>`

Load posts by IDs. Uses `resource::order_as`. Delegates to `new_batch`.

## Helper Functions

### `get_owners(conn, config, posts) -> QueryResult<Vec<Option<MicroUser>>>`

Load post uploaders. Inner-joins `post` with `user`. Returns `(post_id, name, avatar_style)`. Uses `resource::order_like` to align results. Missing owners (posts without uploader) become `None`.

### `get_content_urls(config, posts) -> Result<Vec<String>, Infallible>`

Compute content URLs. Uses `PostHash::new(config, post.id).content_url(post.mime_type)` for each post. The return type `Result<_, Infallible>` guarantees the Result is never Err, matching the `Batcher::exec` signature.

### `get_thumbnail_urls(config, posts) -> Result<Vec<String>, Infallible>`

Compute thumbnail URLs. Uses `PostHash::new(config, post.id).thumbnail_url()` for each post.

### `get_tags(conn, posts) -> QueryResult<Vec<Vec<MicroTag>>>`

Load tags for posts. This is the most complex helper function.

**Step 1 — Load post-tag associations:**
```sql
SELECT PostTag.*, tag.category_id, tag_statistics.usage_count
FROM post_tag
INNER JOIN tag ON ...
INNER JOIN tag_statistics ON ...
INNER JOIN tag_category ON ...
INNER JOIN tag_name ON ...
WHERE post_tag.post_id IN (...)
  AND tag_name.primary = true
ORDER BY tag_category.order, tag_name.name
```

Filters to primary tag names only. Orders by category display order, then name.

**Step 2 — Load all tag names for referenced tags:**
Collects all `tag_id` values from step 1. Queries `tag_name` for all names (including all variants, not only primary). Uses `resource::collect_names` to build a `HashMap<i64, Arc<[SmallString]>>`.

**Step 3 — Load all tag category names:**
Queries `tag_category` table to build a `HashMap<i64, SmallString>`.

**Step 4 — Build MicroTags:**
Groups post_tags by post (using `GroupedBy`). For each group, constructs `MicroTag` objects using the name map, category map, and usage count.

### `get_comments(conn, ctx, posts) -> QueryResult<Vec<Vec<CommentInfo>>>`

Load comments for posts. Loads `(Comment, score, Option<(username, avatar_style)>)` tuples. Also loads own-scores for authenticated user.

**Database query:**
```sql
SELECT Comment.*, comment_statistics.score, (user.name, user.avatar_style)
FROM comment
INNER JOIN comment_statistics ON ...
LEFT JOIN user ON ...
WHERE comment.post_id IN (...)
ORDER BY comment.creation_time
```

**Note:** This function constructs `CommentInfo` directly without using `CommentInfo::new_batch`. It passes `Some(...)` values directly to each field. The `own_score` is resolved from a HashMap built from `comment_score` filtered by client user ID.

### `get_relations(conn, ctx, posts) -> QueryResult<Vec<Vec<MicroPost>>>`

Load related posts. Uses `PostRelation::belonging_to(posts)` ordered by `child_id`. Applies preference filters for hidden posts. Each relation becomes a `MicroPost` with the child post's ID and thumbnail URL.

### `get_pool_launch_dates(conn, posts) -> QueryResult<Vec<Option<DateTime>>>`

Load the earliest pool launch date for each post. Inner-joins `pool_post` with `pool`, filters for non-null `launch_date`. Uses a HashMap to deduplicate — only the first launch date per post is kept.

**Return:** `Vec<Option<DateTime>>` ordered by input post order. Posts without any pool launch date get `None`.

### `get_pools(conn, posts) -> QueryResult<Vec<Vec<MicroPool>>>`

Load pools for posts. Similar complexity to `get_tags`.

**Step 1:** Load `(PoolPost, pool_category_id, post_count)` joined with pool and pool_statistics.

**Step 2:** Collect pool IDs and load names via `collect_names`.

**Step 3:** Load category names as `HashMap<i64, SmallString>`.

**Step 4:** Load pool descriptions and launch dates as `HashMap<i64, (LargeString, Option<DateTime>)>`.

**Step 5:** Group by post, construct `MicroPool` objects.

### `get_notes(conn, posts) -> QueryResult<Vec<Vec<Note>>>`

Load polygon annotations. Uses `PostNote::belonging_to(posts)` ordered by `id`. Each `PostNote` is converted via `Note::new`.

### `get_own_scores(conn, client, posts) -> QueryResult<Vec<Rating>>`

Load authenticated user's scores. If anonymous, returns `Rating::default()` for all. Otherwise queries `post_score` filtered by user ID. Converts `Score` to `Rating` via `Rating::from`.

### `get_own_favorites(conn, client, posts) -> QueryResult<Vec<bool>>`

Check whether authenticated user favorited each post. If anonymous, returns `false` for all. Uses `resource::order_like` with `is_some()` to produce booleans.

### `get_favorited_by(conn, config, posts) -> QueryResult<Vec<Vec<MicroUser>>>`

Load users who favorited each post. Inner-joins `post_favorite` with `user`. Groups by post using `GroupedBy`. Orders users by name.

## Macros

### `post_stats!`

A declarative macro that queries `post_statistics` for a batch of posts.

**Signature:**
```rust
post_stats!($conn, $posts, $column, $return_type)
```

**Parameters:**
- `$conn`: database connection
- `$posts`: slice of Post models
- `$column`: the statistics column to select (from `post_statistics`)
- `$return_type`: the Rust type of the column

**Generated code:**
1. Collect post IDs from the posts slice
2. Query `post_statistics` for those IDs, selecting `(post_id, column)`
3. Use `resource::order_transformed_as` to reorder results by post ID order
4. Map to column values only

**Usage examples in `new_batch`:**
```rust
let mut scores = f.exec(Field::Score, || post_stats!(conn, &posts, score, i64))?;
let mut tag_counts = f.exec(Field::TagCount, || post_stats!(conn, &posts, tag_count, i64))?;
let mut last_feature_times = f.exec(Field::LastFeatureTime, || post_stats!(conn, &posts, last_feature_time, Option<DateTime>))?;
```

**Note:** The macro is declared with `#[doc(hidden)]` and `#[macro_export]` for library use but is not part of the public API documentation.
