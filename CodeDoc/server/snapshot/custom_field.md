# Snapshot — Custom Field

**File:** `server/src/snapshot/custom_field.rs` (62 lines)
**Resource Type:** `ResourceType::CustomField`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `name` | custom string | The display name of the custom field |
| `description` | custom string | Description/help text for the field |
| `color` | custom string | Color associated with the field (e.g., hex or named color) |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

---

## Public Functions

### `creation_snapshot(conn, client, custom_field)`

Called when a new custom field is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `custom_field.id` (converted to string via `to_string()`)
- `data` = full snapshot data (name, description, color)

### `modification_snapshot(conn, client, old, new)`

Called when a custom field is modified.

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
4. If no diff, inserts nothing.

### `deletion_snapshot(conn, client, custom_field)`

Called when a custom field is deleted.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(custom_field) -> Value`

Creates the JSON payload:
```json
{
  "name": <custom_field.name>,
  "description": <custom_field.description>,
  "color": <custom_field.color>
}
```

### `unary_snapshot(conn, client, custom_field, operation)`

Inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::CustomField`
- `resource_id` = `custom_field.id.to_string()` (distinct from other types that use names)
- `data` = `snapshot_data(custom_field)`

---

## Side Effects

One row inserted into the `snapshot` table per call.
No filesystem operations.
No additional database queries.
