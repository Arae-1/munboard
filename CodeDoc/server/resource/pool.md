# Pool Resource — `resource/pool.rs`

**Source:** `server/src/resource/pool.rs` (204 lines)

## Purpose

This file serializes `Pool` model entities into `PoolInfo` API response structs. Pools are ordered collections of posts with names (aliases), a description, and a category. This file also defines `MicroPool`, a lightweight pool representation used by the post resource.

## Dependencies

- `crate::app::Context` — application context (config, client)
- `crate::content::hash::PostHash` — content URL generation
- `crate::model::pool::{Pool, PoolName, PoolPost}` — database models
- `crate::resource` — shared helpers (`single`, `order_as`, `order_transformed_as`)
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::resource::post::MicroPost` — lightweight post representation
- `crate::schema::{pool, pool_category, pool_name, pool_post, pool_statistics}` — Diesel schema
- `crate::search::preferences` — user preference filters
- `crate::string::{LargeString, SmallString}` — string types
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM including `GroupedBy`, `BelongingToDsl`
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `std::sync::Arc` — shared ownership of name slices
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Structs

### `MicroPool`

A lightweight pool representation. Used by `PostInfo` and other resources that embed pool data without the full field set.

| Field | Type | Source | Purpose |
|-------|------|--------|---------|
| `id` | `i64` | Pool ID | Pool identifier (always serialized) |
| `names` | `Arc<[SmallString]>` | `pool_name` table | List of pool names (aliases) |
| `category` | `SmallString` | `pool_category.name` | Category name |
| `description` | `LargeString` | `pool.description` | Markdown description |
| `post_count` | `i64` | `pool_statistics.post_count` | Number of posts in the pool |
| `launch_date` | `Option<DateTime>` | `pool.launch_date` | Launch date or `None` |

**Serialization:** All fields are always present. No field-level selection applies to `MicroPool` — it is always complete.

### `PoolInfo`

The full API response shape for a pool. All fields are `Option` and controlled by the field mask.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `pool.last_edit_time` | Resource version |
| `id` | `Option<i64>` | `pool.id` | Pool identifier |
| `description` | `Option<LargeString>` | `pool.description` | Markdown content |
| `creation_time` | `Option<DateTime>` | `pool.creation_time` | When created |
| `last_edit_time` | `Option<DateTime>` | `pool.last_edit_time` | When last edited |
| `category` | `Option<SmallString>` | Joined from `pool_category` | Category name |
| `names` | `Option<Vec<SmallString>>` | From `pool_name` grouped by pool | Ordered list of aliases |
| `posts` | `Option<Vec<MicroPost>>` | From `pool_post` | Ordered list of posts in the pool |
| `post_count` | `Option<i64>` | `pool_statistics.post_count` | Total post count |
| `launch_date` | `Option<Option<DateTime>>` | `pool.launch_date` | Double-Option: `None` = field not requested; `Some(None)` = field requested but no launch date |

## Field Enum

```rust
pub enum Field {
    Version, Id, Description, CreationTime, LastEditTime,
    Category, Names, Posts, PostCount, LaunchDate,
}
```

## Constructor Methods

### `PoolInfo::new(conn, ctx, pool, fields) -> QueryResult<Self>`

Create a single pool resource. Delegates to `new_batch` and calls `resource::single`.

### `PoolInfo::new_from_id(conn, ctx, pool_id, fields) -> QueryResult<Self>`

Load from ID, then delegate to `new_batch`.

### `PoolInfo::new_batch(conn, ctx, pools, fields) -> QueryResult<Vec<Self>>`

Create multiple pool resources. Uses `Batcher` for conditional loading.

**Conditionally loaded data:**

| Field | Helper Function | Data Source |
|-------|----------------|-------------|
| `Category` | `get_categories` | `pool` INNER JOIN `pool_category` |
| `Names` | `get_names` | `PoolName::belonging_to(pools)` grouped with `GroupedBy` |
| `Posts` | `get_posts` | `PoolPost::belonging_to(pools)` filtered by preferences |
| `PostCount` | `get_post_counts` | `pool_statistics` |

**Construction:** Uses reverse-iteration and pop pattern, same as all other resources.

### `PoolInfo::new_batch_from_ids(conn, ctx, pool_ids, fields) -> QueryResult<Vec<Self>>`

Load pools by IDs. Uses `resource::order_as`. Delegates to `new_batch`.

## Helper Functions

### `get_categories(conn, pools) -> QueryResult<Vec<SmallString>>`

Load category names for pools. Inner-joins `pool` with `pool_category`. Returns `(pool.id, pool_category.name)` tuples ordered by input pool order via `resource::order_transformed_as`.

### `get_names(conn, pools) -> QueryResult<Vec<Vec<SmallString>>>`

Load pool names. Uses `PoolName::belonging_to(pools)` ordered by `pool_name::order`. Groups results by pool using `GroupedBy`. Maps each group to a `Vec<SmallString>`.

### `get_posts(conn, ctx, pools) -> QueryResult<Vec<Vec<MicroPost>>>`

Load posts in each pool. Uses `PoolPost::belonging_to(pools)` ordered by `pool_post::order`.

**Preference filtering:** If the client has hidden posts configured, adds `WHERE NOT EXISTS (hidden_post_filter)` to the query.

**Transform:** Each `PoolPost` becomes a `MicroPost` with `id = pool_post.post_id` and `thumbnail_url` from `PostHash`.

### `get_post_counts(conn, pools) -> QueryResult<Vec<i64>>`

Load post counts from `pool_statistics`. Uses `resource::order_transformed_as` to align with input order.
