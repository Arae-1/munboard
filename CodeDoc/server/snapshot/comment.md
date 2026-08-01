# Snapshot — Comment

**File:** `server/src/snapshot/comment.rs` (61 lines)
**Resource Type:** `ResourceType::Comment`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `text` | custom string | The comment body text |
| `post_id` | `i64` | The ID of the post this comment belongs to |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

---

## Public Functions

### `creation_snapshot(conn, client, comment)`

Called when a new comment is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `comment.id` (converted to `SmallString`)
- `data` = full snapshot data (text, post_id)

Delegates to `unary_snapshot`.

### `modification_snapshot(conn, client, old, new)`

Called when an existing comment is modified.

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both `old` and `new` to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `NewSnapshot` with:
   - `operation` = `ResourceOperation::Modified`
   - `resource_id` = `old.id`
   - `data` = the diff
4. If no diff exists, inserts nothing.

### `deletion_snapshot(conn, client, comment)`

Called when a comment is deleted.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(comment) -> Value`

Creates the JSON payload:
```json
{
  "text": <comment.text>,
  "post_id": <comment.post_id>
}
```

### `unary_snapshot(conn, client, comment, operation)`

Creates and inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::Comment`
- `resource_id` = `comment.id`
- `data` = `snapshot_data(comment)`

Returns `QueryResult<()>` from the insert.

---

## Side Effects

Inserts one row into the `snapshot` database table per call.
No filesystem operations.
No additional database queries beyond the insert.
