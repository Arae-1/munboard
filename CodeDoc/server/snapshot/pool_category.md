# Snapshot — Pool Category

**File:** `server/src/snapshot/pool_category.rs` (77 lines)
**Resource Type:** `ResourceType::PoolCategory`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `name` | custom string | The display name of the pool category |
| `color` | custom string | Color associated with the category |
| `default` | `bool` | Computed as `pool_category.id == 0` — whether this is the default category |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

---

## Public Functions

### `creation_snapshot(conn, client, pool_category)`

Called when a new pool category is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `pool_category.name` (uses the name string as identifier)
- `data` = full snapshot data

### `set_default_snapshot(conn, client, old_default, new_default)`

Called when the default pool category is changed.

Delegates to `snapshot::set_default_snapshot` with:
- `old_default_name` = `old_default.name`
- `new_default_name` = `new_default.name`
- `resource_type` = `ResourceType::PoolCategory`

This creates two `Modified` snapshots:
- One marking the old default as `{"default": false}`
- One marking the new default as `{"default": true}`

If `old_default.name == new_default.name`, no snapshots are created.

### `modification_snapshot(conn, client, old, new)`

Called when a pool category is modified.

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
4. If no diff, inserts nothing.

**Note:** `resource_id` uses `old.name` (the name string), not the ID.

### `deletion_snapshot(conn, client, pool_category)`

Called when a pool category is deleted.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(pool_category) -> Value`

Creates the JSON payload:
```json
{
  "name": <pool_category.name>,
  "color": <pool_category.color>,
  "default": <pool_category.id == 0>
}
```

The `default` field is computed as a boolean: `true` when the category ID is 0.

### `unary_snapshot(conn, client, pool_category, operation)`

Inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::PoolCategory`
- `resource_id` = `pool_category.name` (the name string)
- `data` = `snapshot_data(pool_category)`

---

## Side Effects

One row inserted into the `snapshot` table per call.
For `set_default_snapshot`, up to two rows inserted.
No filesystem operations.
No additional database queries beyond the insert.
