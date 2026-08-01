# Pool API — `pool.rs`

File: `server/src/api/pool.rs`

## Overview

This module defines all pool-related HTTP endpoints. It handles CRUD operations, search, post membership management, and pool merging.

---

## Router Registration

### `routes() -> OpenApiRouter<AppState>`

**Location:** `pool.rs:25`

Returns a router with these endpoints:

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/pools` | `list` |
| `GET` | `/pool/{id}` | `get` |
| `POST` | `/pool` | `create` |
| `POST` | `/pool-merge` | `merge` |
| `PUT` | `/pool/{id}` | `update` |
| `DELETE` | `/pool/{id}` | `delete` |

---

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_POOLS_PER_PAGE` | `1000` | Maximum pools per page in list queries |

---

## Endpoints

### 1. `GET /pools` — List Pools

**Handler:** `list` at `pool.rs:75`

**Authentication:** Requires `Action::PoolList`.

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
| `name` | Pool name (wildcards allowed) |
| `category` | Pool category (wildcards allowed) |
| `creation-date`, `creation-time` | Creation date |
| `last-edit-date`, `last-edit-time`, `edit-date`, `edit-time` | Last edit date |
| `post-count` | Number of posts in pool |

**Sort Tokens:** `random`, `name`, `category`, `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time`, `post-count`

**Successful Response:** `200` with `PagedResponse<PoolInfo>`

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::PoolList` privilege.
2. Cap `limit` at `MAX_POOLS_PER_PAGE` (1000).
3. Open a database transaction.
4. Build `QueryBuilder` from the search query.
5. Set offset and limit.
6. Execute `query_builder.list(conn)` to get `(total, selected_pool_ids)`.
7. Batch-load `PoolInfo` from the selected pool IDs.
8. Return `PagedResponse` as JSON.

---

### 2. `GET /pool/{id}` — Get Single Pool

**Handler:** `get` at `pool.rs:116`

**Authentication:** Requires `Action::PoolView`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Pool ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PoolInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Pool does not exist

**Handler Logic:**
1. Verify `Action::PoolView` privilege.
2. Open a database transaction.
3. Check if the pool exists (`exists(pool::table.find(pool_id))`). Return `NotFound` if absent.
4. Load `PoolInfo` from the pool ID with requested fields.
5. Return as JSON.

---

### 3. `POST /pool` — Create a Pool

**Handler:** `create` at `pool.rs:175`

**Authentication:** Requires `Action::PoolCreate`.

**Request Body:** `PoolCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `names` | `Vec<SmallString>` | Pool names (must match `pool_name_regex`) |
| `category` | `SmallString` | Category name (must exist) |
| `description` | `Option<LargeString>` | Pool description |
| `launchDate` | `Option<DateTime>` | Launch date |
| `posts` | `Option<Vec<i64>>` | Initial post IDs |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PoolInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — At least one post ID does not exist
- `409` — Any name is used by an existing pool
- `409` — Duplicate posts in the post list
- `422` — No names given (`ApiError::NoNamesGiven`)
- `422` — A name is invalid
- `422` — Category missing or invalid

**Handler Logic:**
1. Verify `Action::PoolCreate` privilege.
2. If `body.names` is empty, return `ApiError::NoNamesGiven`.
3. Open a database transaction.
4. Look up the category by name in `pool_category::table`. Return `NotFound` if absent.
5. Insert `NewPool` into `pool::table`.
6. Set names via `update::pool::set_names`.
7. Add initial posts via `update::pool::add_posts(pool_id, starting_position=0, posts)`.
8. Record a `creation_snapshot` with pool data (description, launch_date, category, names, posts).
9. After the first transaction, load `PoolInfo` and return as JSON.

---

### 4. `POST /pool-merge` — Merge Pools

**Handler:** `merge` at `pool.rs:245`

**Authentication:** Requires `Action::PoolMerge`.

**Request Body:** `MergeBody<i64>`

| Field | Type | Description |
|-------|------|-------------|
| `remove` | `i64` | Source pool ID to remove |
| `mergeTo` | `i64` | Target pool ID to merge into |
| `removeVersion` | `DateTime` | Version of source pool |
| `mergeToVersion` | `DateTime` | Version of target pool |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PoolInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Source or target pool does not exist
- `409` — Version outdated
- `422` — Source equals target (`ApiError::SelfMerge`)

**Handler Logic:**
1. Verify `Action::PoolMerge` privilege.
2. If `remove == merge_to`, return `ApiError::SelfMerge`.
3. Define `get_pool_info` closure: selects `last_edit_time` from pool by ID.
4. Open a database transaction.
5. Look up the source pool. Return `NotFound` if absent.
6. Look up the target pool. Return `NotFound` if absent.
7. Verify versions of both.
8. Call `update::pool::merge` to merge posts and aliases from source to target.
9. Record a `merge_snapshot`.
10. After the first transaction, load `PoolInfo` for the target and return as JSON.

---

### 5. `PUT /pool/{id}` — Update a Pool

**Handler:** `update` at `pool.rs:339`

**Authentication:** Different privileges for each field: `PoolEditCategory`, `PoolEditDescription`, `PoolEditLaunchDate`, `PoolEditName`, `PoolEditPost`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Pool ID |

**Request Body:** `PoolUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version (required) |
| `category` | `Option<SmallString>` | New category name |
| `description` | `Option<LargeString>` | New description |
| `launchDate` | `Option<Option<DateTime>>` | Launch date (send `null` to clear) |
| `names` | `Option<Vec<SmallString>>` | New names |
| `posts` | `Option<Vec<i64>>` | New post list (replaces all) |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PoolInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Pool does not exist or a post ID does not exist
- `409` — Version outdated, name clash, duplicate posts
- `422` — Name invalid, no names, category invalid

**Handler Logic:**
1. Open a database transaction.
2. Load the existing pool. Return `NotFound` if absent.
3. Verify version.
4. Clone the pool and retrieve old snapshot data. Clone the snapshot data.
5. For each provided field:
   - **category:** Verify `PoolEditCategory`, look up category ID, update `category_id` and snapshot.
   - **description:** Verify `PoolEditDescription`, update description on snapshot.
   - **launch_date:** Verify `PoolEditLaunchDate`, update on pool and snapshot (supports `null` to clear via `deserialize_clearable_date`).
   - **names:** Verify `PoolEditName`. If empty, return `ApiError::NoNamesGiven`. Call `update::pool::set_names`.
   - **posts:** Verify `PoolEditPost`. Call `update::pool::set_posts`, which validates post IDs and replaces the entire list.
6. Update `last_edit_time` to now.
7. Save the pool via `save_changes`.
8. Record a `modification_snapshot` with old and new snapshot data.
9. After the first transaction, load `PoolInfo` and return as JSON.

---

### 6. `DELETE /pool/{id}` — Delete a Pool

**Handler:** `delete` at `pool.rs:429`

**Authentication:** Requires `Action::PoolDelete`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Pool ID |

**Request Body:** `DeleteBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |

**Successful Response:** `200` with `()`

**Error Responses:**
- `403` — Privileges too low
- `404` — Pool does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Verify `Action::PoolDelete` privilege.
2. Open a database transaction.
3. Load the pool. Return `NotFound` if absent.
4. Verify version.
5. Retrieve snapshot data for the pool.
6. Record a `deletion_snapshot`.
7. Delete the pool from `pool::table`.
8. Posts in the pool only lose their relation to the pool (cascading effect). The pool record is removed.
9. Return `Json(())`.

---

## Type Definitions

### `PoolCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `names` | `Vec<SmallString>` | Pool names |
| `category` | `SmallString` | Category name |
| `description` | `Option<LargeString>` | Pool description |
| `launchDate` | `Option<DateTime>` | Launch date |
| `posts` | `Option<Vec<i64>>` | Initial post IDs |

### `PoolUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |
| `category` | `Option<SmallString>` | New category |
| `description` | `Option<LargeString>` | New description |
| `launchDate` | `Option<Option<DateTime>>` | Launch date (null to clear) |
| `names` | `Option<Vec<SmallString>>` | New names |
| `posts` | `Option<Vec<i64>>` | New post list (replaces all) |
