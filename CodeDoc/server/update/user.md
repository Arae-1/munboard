# Update — User

**File:** `server/src/update/user.rs` (40 lines)
**Purpose:** User login time tracking, edit time tracking, custom avatar management.

---

## Public Functions

### `last_login_time(conn, user_id) -> QueryResult<()>`

Records the current time as the user's last login time.

```sql
UPDATE user SET last_login_time = NOW() WHERE id = user_id
```

Called when a user successfully authenticates. Returns `QueryResult<()>` — no error mapping.

**Side effect:** One database UPDATE. No privilege check.

---

### `last_edit_time(conn, user_id) -> QueryResult<()>`

Updates the `last_edit_time` of the user to the current time.

```sql
UPDATE user SET last_edit_time = NOW() WHERE id = user_id
```

Called when the user's profile is modified. Returns `QueryResult<()>` — no error mapping.

**Side effect:** One database UPDATE. No privilege check.

---

### `avatar(conn, config, user_id, name, avatar) -> ApiResult<()>`

Updates the custom avatar for a user.

**Parameters:**
- `user_id`: `i64` — the user's database ID
- `name`: `&str` — the user's name (used for filesystem path)
- `avatar`: `&DynamicImage` — the new avatar image

**Steps:**

1. **Delete old avatar:** `filesystem::delete_custom_avatar(config, name)` — removes the existing custom avatar file from disk. If no avatar exists, this is a no-op.

2. **Save new avatar:** `filesystem::save_custom_avatar(config, name, avatar)` — writes the new avatar to disk. Returns the file size in bytes.

3. **Update database:**
   ```sql
   UPDATE user SET custom_avatar_size = <size> WHERE id = user_id
   ```

**Side effects:**
- Filesystem write (save new avatar)
- Filesystem delete (remove old avatar)
- One database UPDATE

---

## Side Effects Summary

| Function | Database | Filesystem | Privilege Check |
|---|---|---|---|
| `last_login_time` | UPDATE user | — | No |
| `last_edit_time` | UPDATE user | — | No |
| `avatar` | UPDATE user | Delete + Save avatar file | No (caller must check) |

All three functions are lightweight. The `avatar` function is the only one with filesystem side effects.
