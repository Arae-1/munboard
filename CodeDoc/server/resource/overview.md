# Resource Layer

> **Detailed documentation:** See individual files in this directory — [mod.md](mod.md), [field.md](field.md), [comment.md](comment.md), [custom_field.md](custom_field.md), [pool.md](pool.md), [pool_category.md](pool_category.md), [post.md](post.md), [snapshot.md](snapshot.md), [tag.md](tag.md), [tag_category.md](tag_category.md), [user.md](user.md), [user_token.md](user_token.md).

## Purpose

The resource module provides the API response serialization layer. Each resource type has a corresponding `*Info` struct that represents the JSON shape returned to clients. Resources support field-level selection via a bitmask, lazy batch loading of related data, and ordered result assembly.

## Field Selection System (`field.rs`)

### `Mask<F>`
A u64 bitmask where each bit represents a requestable field. Each resource type defines its own `Field` enum with `Into<u64>` conversion.

### Operations
- `Mask::new()`: Empty mask (no fields selected).
- `Mask::from_u64(value)`: Mask from raw bits.
- `contains(field)`: Returns true if the field is selected.
- `Index<F>`: Returns `&true` or `&false`.
- `BitOr<F>` / `BitOrAssign<F>`: Add a field to the mask.
- `From<&[F]>` / `From<[F; N]>`: Builds a mask from a list of fields.
- Deserialize: Parses a comma-separated field list from a query string. An absent/null value means "all fields" (u64::MAX).

### `Batcher<F>`
Lazy batch loader for related data. Constructor takes the field mask and batch size.

### `exec(field, function)`
Conditionally executes a data-loading function:
- If the field is in the mask, calls the function and verifies the result length matches batch size.
- If not, returns an empty Vec.
- This avoids unnecessary database queries for unrequested fields.

## Shared Utilities (`mod.rs`)

### `single(batch)`
Extracts exactly one element from a batch, panicking if the batch size is not 1.

### `order_as(values, order)`
Reorders a Vec of Identifiable items so their IDs match the order slice. Uses O(n^2) algorithm (suitable for small batches of 40-50). Panics if values are not unique.

### `order_transformed_as(values, order, get_id)`
Same as `order_as` but uses a custom function to extract IDs.

### `order_like(unordered_values, ordered_values, get_id)`
Maps unordered values into an Option vector ordered by reference values. Missing values become None. Useful when eager-loading related data that may not exist for every parent.

### `collect_names(ordered_names)`
Builds a HashMap from resource ID to Arc<[SmallString]> from a sorted list of (id, name) tuples. Detects boundaries where consecutive names belong to different resources. Minimizes allocations by pre-allocating capacity and using Arc for shared ownership.

## Comment Resource (`comment.rs`)

### `Field` Enum
`Version`, `Id`, `PostId`, `Text`, `CreationTime`, `LastEditTime`, `User`, `Score`, `OwnScore`.

### `CommentInfo`
Fields:
- `version`: Comment's `last_edit_time`.
- `id`: Comment ID.
- `post_id`: Parent post ID.
- `text`: Markdown content.
- `creation_time`, `last_edit_time`: Timestamps.
- `user`: Optional MicroUser (null for anonymous).
- `score`: Collective score from `comment_statistics`.
- `own_score`: Authenticated user's rating, or `Rating::default()`.

### Batch Loading
Supports single, batch-from-models, and batch-from-IDs construction. Uses `Batcher` to conditionally load:
- **User**: Joins comment with user table.
- **Score**: From `comment_statistics`.
- **OwnScore**: From `comment_score` filtered by client ID.

## Custom Field Resource (`custom_field.rs`)

### `CustomFieldValue`
A simple value struct with `name`, `value` (i64), and `color`.

### `Field` Enum
`Version`, `Id`, `Name`, `Description`, `Color`, `CreationTime`.

### `CustomFieldInfo`
Fields: `version`, `id`, `name`, `description`, `color`, `creation_time`.

### `get_custom_fields_for_posts`
Bulk-loads custom field values for multiple posts. Joins `post_custom_field` with `custom_field`. Returns a `Vec<Vec<CustomFieldValue>>` ordered by the input post IDs.

## Pool Resource (`pool.rs`)

### `MicroPool`
A lightweight pool representation: `id`, `names` (Arc<[SmallString]>), `category`, `description`, `post_count`, `launch_date`.

### `Field` Enum
`Version`, `Id`, `Description`, `CreationTime`, `LastEditTime`, `Category`, `Names`, `Posts`, `PostCount`, `LaunchDate`.

### `PoolInfo`
Full pool resource with conditional fields. Batch loading:
- **Category**: Joins `pool` with `pool_category`.
- **Names**: Uses `PoolName::belonging_to` grouped by pool, ordered by `order`.
- **Posts**: Uses `PoolPost::belonging_to`, filters hidden posts via preferences, returns `MicroPost` (id + thumbnail_url).
- **PostCount**: From `pool_statistics`.

## Pool Category Resource (`pool_category.rs`)

### `Field` Enum
`Version`, `Name`, `Color`, `Usages`, `Default`.

### `PoolCategoryInfo`
Fields: `version`, `name`, `color`, `usages` (from pool_category_statistics), `default` (true if id == 0).

### `all`
Loads all pool categories ordered by ID, joined with statistics.

## Post Resource (`post.rs`)

### `Note`
A polygon annotation for a post:
- `id`: Internal ID (not serialized).
- `polygon`: Vec of `[f32; 2]` vertex coordinates in [0, 1] range.
- `text`: Markdown annotation text.

`Note::new` converts from `PostNote` model, chunking the flat polygon array into coordinate pairs. `to_new_post_note` reverses this for insertion.

### `MicroPost`
Lightweight post: `id`, `thumbnail_url`.

### `Field` Enum
30 fields covering all post metadata, relations, statistics, and user-specific data.

### `PostInfo`
Full post resource. Batch loading covers:
- **User**: Post owner (MicroUser, nullable).
- **ContentUrl**: Via PostHash.
- **ThumbnailUrl**: Via PostHash.
- **Tags**: MicroTags with names, category, and usage stats. Joins post_tag -> tag -> tag_statistics -> tag_category -> tag_name. Names and categories are bulk-loaded via collect_names.
- **Comments**: Full CommentInfo objects with scores and own_scores.
- **Relations**: MicroPosts filtered by hidden post preferences.
- **Pools**: MicroPools with names, category, description, post_count, launch_date.
- **PoolLaunchDates**: Earliest pool launch date for each post.
- **Notes**: Full Note objects.
- **Statistics**: Score, tag_count, comment_count, relation_count, note_count, favorite_count, feature_count, last_feature_time — all from `post_statistics` via the `post_stats!` macro.
- **OwnScore**: Authenticated user's rating.
- **OwnFavorite**: Whether authenticated user favorited the post.
- **FavoritedBy**: List of MicroUsers who favorited the post.
- **HasCustomThumbnail**: Checks filesystem for custom thumbnail presence.
- **CustomFields**: From `get_custom_fields_for_posts`.

### `post_stats!` macro
A helper macro that queries `post_statistics` for a batch of posts and returns the values ordered by post ID.

## Snapshot Resource (`snapshot.rs`)

### `Field` Enum
`User`, `Operation`, `Type`, `Id`, `Data`, `Time`.

### `SnapshotInfo`
Represents a versioned change to a database resource:
- `user`: Optional MicroUser who made the change.
- `operation`: Created, Modified, Deleted, or Merged.
- `resource_type` (serialized as "type"): Tag, tag_category, post, pool, or pool_category.
- `resource_id` (serialized as "id"): The resource identifier.
- `data`: JSON diff for modifications, full data for create/delete, tuple for merges.
- `time`: When the snapshot was created.

## Tag Resource (`tag.rs`)

### `MicroTag`
Lightweight tag: `names` (Arc<[SmallString]>), `category`, `usages`.

### `Field` Enum
`Version`, `Description`, `CreationTime`, `LastEditTime`, `Category`, `Names`, `Implications`, `Suggestions`, `Usages`.

### `TagInfo`
Full tag resource. Batch loading:
- **Category**: Category name via join.
- **Names**: Ordered list from tag_name, grouped by tag.
- **Implications**: MicroTags with names, category, usages. Loads names and categories via collect_names.
- **Suggestions**: Same structure as implications.
- **Usages**: From tag_statistics.

## Tag Category Resource (`tag_category.rs`)

### `Field` Enum
`Version`, `Name`, `Color`, `Usages`, `Order`, `Default`.

### `TagCategoryInfo`
Fields: `version`, `name`, `color`, `usages`, `order` (display order), `default` (id == 0).

### `all`
Loads all tag categories ordered by `order`, joined with statistics. Filters out blacklisted categories for anonymous users.

## User Resource (`user.rs`)

### `MicroUser`
Lightweight user: `name`, `avatar_url` (Gravatar or custom).

### `Visibility`
- `Full`: All fields visible (for the authenticated user's own profile or admins).
- `PublicOnly`: Private fields hidden.

### `Field` Enum
`Version`, `Name`, `Email`, `Rank`, `LastLoginTime`, `CreationTime`, `AvatarStyle`, `AvatarUrl`, `CommentCount`, `UploadedPostCount`, `LikedPostCount`, `DislikedPostCount`, `FavoritePostCount`.

### `UserInfo`
Full user resource. Private data fields use a `PrivateData<T>` enum:
- `Expose(T)`: The actual value.
- `Visible(bool)`: Always false, indicating data is hidden.

Batch loading:
- **AvatarUrl**: From user model.
- **Email**: Exposed for Full visibility, hidden (false) for PublicOnly.
- **Statistics**: comment_count, upload_count, favorite_count from user_statistics via `user_stats!` macro.
- **Liked/Disliked counts**: Counts from post_score with score = Like/Dislike. Hidden for PublicOnly visibility.

### `user_stats!` macro
Same pattern as `post_stats!`, queries user_statistics for a batch of users.

## User Token Resource (`user_token.rs`)

### `Field` Enum
`Version`, `User`, `Note`, `Enabled`, `ExpirationTime`, `CreationTime`, `LastEditTime`, `LastUsageTime`, `Token`.

### `Visibility`
- `Full`: The token UUID is visible.
- `PublicOnly`: The token UUID is never visible.

### `UserTokenInfo`
Single token construction (no batch loading). Fields:
- `version`: last_edit_time.
- `user`: MicroUser (the token owner).
- `note`: Description.
- `enabled`: Whether active.
- `expiration_time`: Optional expiry.
- `creation_time`, `last_edit_time`, `last_usage_time`: Timestamps.
- `token`: UUID string, only visible when field is selected AND visibility is Full.
