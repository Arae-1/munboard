# Snapshot — Pool

**File:** `server/src/snapshot/pool.rs` (133 lines)
**Resource Type:** `ResourceType::Pool`

---

## Struct: `SnapshotData`

A data container that represents the complete snapshot state of a pool.

### Fields

| Field | Type | Description |
|---|---|---|
| `description` | `LargeString` | Text description of the pool |
| `launch_date` | `Option<DateTime>` | Optional date when the pool was published/launched |
| `category` | `SmallString` | Name of the pool category |
| `names` | `Vec<SmallString>` | Ordered list of pool names (aliases) |
| `posts` | `Vec<i64>` | Ordered list of post IDs in the pool |

Derives: `Clone`, `Serialize`.

### Method: `retrieve(conn, pool) -> QueryResult<Self>`

Takes a `Pool` model and queries the database for related data. Performs **three database queries**:

1. **Category name lookup:**
   ```sql
   SELECT pool_category.name FROM pool_category WHERE id = pool.category_id
   ```

2. **Names lookup:**
   ```sql
   SELECT pool_name.name FROM pool_name WHERE pool_id = pool.id
   ```

3. **Posts lookup:**
   ```sql
   SELECT pool_post.post_id FROM pool_post WHERE pool_id = pool.id
   ```

Returns a fully populated `SnapshotData`.

### Method: `sort_fields(&mut self)`

Sorts `names` and `posts` vectors in-place using `sort_unstable()`. This ensures deterministic ordering before diff computation, so identical sets of names/posts in different orders do not produce false diffs.

---

## Public Functions

### `creation_snapshot(conn, client, pool_id, pool_data) -> ApiResult<()>`

Called when a new pool is created. Takes a pre-built `SnapshotData`.

Calls `unary_snapshot` with `ResourceOperation::Created`.

### `merge_snapshot(conn, client, absorbed_pool_id, merge_to_pool_id) -> QueryResult<()>`

Called when one pool is merged into another.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Merged`
- `resource_id` = `absorbed_pool_id`
- `data` = `[ResourceType::Pool, merge_to_pool_id]` (a JSON array indicating the merge target)

This is distinct from other submodules — `merge_snapshot` does NOT use unary_snapshot or value_diff. It directly builds a JSON array payload.

### `modification_snapshot(conn, client, pool_id, old, new) -> ApiResult<()>`

Called when a pool is modified.

1. Calls `sort_fields()` on both `old` and `new` data.
2. Serializes both to JSON via `serde_json::to_value`.
3. Calls `snapshot::value_diff(old_data, new_data)`.
4. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
5. If no diff, inserts nothing.

Uses `ApiResult` — serialization errors (from `serde_json`) are returned as `ApiError`.

### `deletion_snapshot(conn, client, pool_id, pool_data) -> ApiResult<()>`

Called when a pool is deleted.

Calls `unary_snapshot` with `ResourceOperation::Deleted`.

---

## Private Function: `unary_snapshot(conn, client, pool_id, pool_data, operation) -> ApiResult<()>`

1. Calls `sort_fields()` on `pool_data`.
2. Serializes data to JSON with `serde_json::to_value`. Maps `serde_json::Error` to `ApiError`.
3. Creates and inserts a `NewSnapshot`:
   - `user_id` = `client.id`
   - `operation` = the given operation
   - `resource_type` = `ResourceType::Pool`
   - `resource_id` = `pool_id` (converted to `SmallString`)
   - `data` = serialized JSON
4. Maps insert errors to `ApiError`.

---

## Side Effects

- `retrieve` performs 3 database reads
- Each snapshot function inserts 1 row into the `snapshot` table
- `sort_fields` mutates the data in-place
- No filesystem operations
