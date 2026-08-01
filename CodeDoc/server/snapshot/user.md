# Snapshot — User

**File:** `server/src/snapshot/user.rs` (62 lines)
**Resource Type:** `ResourceType::User`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `name` | custom string | The username/login name |
| `rank` | custom string | The user's privilege rank (e.g., administrator, moderator, regular) |
| `avatar_style` | ? | The user's avatar style setting (determines how the avatar is displayed) |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

**Note:** The user's `id` field is NOT tracked in the snapshot data, only the `name` is used. The `resource_id` of the snapshot is set to `user.name`.

---

## Public Functions

### `creation_snapshot(conn, client, user)`

Called when a new user account is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `user.name` (the username string)
- `data` = full snapshot data (name, rank, avatar_style)

### `modification_snapshot(conn, client, old, new)`

Called when a user's profile is modified.

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
4. If no diff, inserts nothing.

**Note:** `resource_id` is `old.name`. If the username was changed, the snapshot is recorded under the old name.

### `deletion_snapshot(conn, client, user)`

Called when a user account is deleted.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(user) -> Value`

Creates the JSON payload:
```json
{
  "name": <user.name>,
  "rank": <user.rank>,
  "avatar_style": <user.avatar_style>
}
```

### `unary_snapshot(conn, client, user, operation)`

Inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::User`
- `resource_id` = `user.name` (uses name, not ID)
- `data` = `snapshot_data(user)`

---

## Side Effects

One row inserted into the `snapshot` table per call.
No filesystem operations.
No additional database queries beyond the insert.
