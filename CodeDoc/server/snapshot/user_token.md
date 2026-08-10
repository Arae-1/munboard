# Snapshot — User Token

**File:** `server/src/snapshot/user_token.rs` (62 lines)
**Resource Type:** `ResourceType::UserToken`

---

## Tracked Fields

| Field | Type | Description |
|---|---|---|
| `note` | custom string | A note/label describing the purpose of the token |
| `enabled` | `bool` | Whether the token is currently active/enabled |
| `expiration_time` | `Option<DateTime>` | When the token expires. `DateTime` serializes as an RFC 3339 timestamp string. Tokens created through the API always carry an expiration (a 30-day default when none is given), so creation snapshots record a timestamp, never `null` |

Fields are serialized via inline `json!({...})` in the private `snapshot_data` function.

**Note:** The actual token value is NOT tracked — only metadata about the token.

---

## Public Functions

### `creation_snapshot(conn, client, user_token)`

Called when a new user token (API key) is created.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Created`
- `resource_id` = `user_token.id.to_string()` (the numeric ID as string)
- `data` = full snapshot data (note, enabled, expiration_time)

The API create handler gives tokens without an explicit expiration a default lifetime of 30 days. The snapshot therefore records `expiration_time` as an RFC 3339 timestamp, never `null`.

### `modification_snapshot(conn, client, old, new)`

Called when a user token is modified (e.g., note changed, enabled toggled).

**Precondition:** `old.id == new.id` (asserted).

1. Serializes both to JSON via `snapshot_data`.
2. Calls `snapshot::value_diff(old_data, new_data)`.
3. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
4. If no diff, inserts nothing.

### `deletion_snapshot(conn, client, user_token)`

Called when a user token/API key is revoked.

Same as creation but with `operation` = `ResourceOperation::Deleted`.

---

## Private Functions

### `snapshot_data(user_token) -> Value`

Creates the JSON payload:
```json
{
  "note": <user_token.note>,
  "enabled": <user_token.enabled>,
  "expiration_time": <user_token.expiration_time>
}
```

### `unary_snapshot(conn, client, user_token, operation)`

Inserts a single `NewSnapshot` with:
- `user_id` = `client.id`
- `operation` = the given operation
- `resource_type` = `ResourceType::UserToken`
- `resource_id` = `user_token.id.to_string()` (numeric ID as string)
- `data` = `snapshot_data(user_token)`

---

## Side Effects

One row inserted into the `snapshot` table per call.
No filesystem operations.
No additional database queries beyond the insert.
