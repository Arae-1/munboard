# Snapshot — Tag Category

**File:** `server/src/snapshot/tag_category.rs` (77 lines)
**Resource Type:** `ResourceType::TagCategory`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `name` | custom string | The display name of the tag category |
| `color` | custom string | Color associated with the category |
| `default` | `bool` | Computed as `tag_category.id == 0` — whether this is the default category |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

---

## Public Functions

### `creation_snapshot(conn, client, tag_category)`

Called when a new tag category is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `tag_category.name` (uses the name string as identifier)
- `data` = full snapshot data (name, color, default boolean)

### `set_default_snapshot(conn, client, old_default, new_default)`

Called when the default tag category is changed.

Delegates to `snapshot::set_default_snapshot` with:
- `old_default_name` = `old_default.name`
- `new_default_name` = `new_default.name`
- `resource_type` = `ResourceType::TagCategory`

This creates two `Modified` snapshots:
- One marking the old default as `{"default": false}`
- One marking the new default as `{"default": true}`

If `old_default.name == new_default.name`, no snapshots are created.

### `modification_snapshot(conn, client, old, new)`

Called when a tag category is modified.

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
4. If no diff, inserts nothing.

**Note:** `resource_id` uses `old.name` (the name string), not the ID. This means if the name changes, the resource_id changes for the snapshot.

### `deletion_snapshot(conn, client, tag_category)`

Called when a tag category is deleted.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(tag_category) -> Value`

Creates the JSON payload:
```json
{
  "name": <tag_category.name>,
  "color": <tag_category.color>,
  "default": <tag_category.id == 0>
}
```

The `default` field is a computed boolean: `true` when the category ID is 0.

### `unary_snapshot(conn, client, tag_category, operation)`

Inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::TagCategory`
- `resource_id` = `tag_category.name` (the name string)
- `data` = `snapshot_data(tag_category)`

---

## Side Effects

One row inserted into the `snapshot` table per call.
For `set_default_snapshot`, up to two rows inserted.
No filesystem operations.
No additional database queries beyond the insert.
