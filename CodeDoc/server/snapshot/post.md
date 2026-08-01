# Snapshot — Post

**File:** `server/src/snapshot/post.rs` (207 lines)
**Resource Type:** `ResourceType::Post`

---

## Struct: `SnapshotData`

The most complex snapshot data in the system. Represents the complete state of a post at the time of snapshot.

### Fields

| Field | Type | Description |
|---|---|---|
| `safety` | `PostSafety` | Post safety rating (Safe, Sketchy, Unsafe) |
| `checksum` | `String` | Hex-encoded content checksum |
| `flags` | `PostFlags` | Bitfield of post flags |
| `source` | `LargeString` | Source URL or attribution |
| `description` | `LargeString` | Post description text |
| `launch_date` | `Option<DateTime>` | Optional date the post was first published/launched |
| `media_created_date` | `Option<DateTime>` | Optional date the media file was created |
| `tags` | `Vec<SmallString>` | List of primary tag names on the post |
| `relations` | `Vec<i64>` | List of related post IDs (children) |
| `notes` | `Vec<Note>` | List of post annotations (polygon notes) |
| `featured` | `bool` | Whether this post is currently the featured post |
| `custom_fields` | `Vec<CustomFieldValue>` | Custom field name/value/color tuples |

Derives: `Clone`, `Serialize`.

### Method: `retrieve(conn, post) -> QueryResult<Self>`

Takes a `Post` model and queries the database for all related data. Performs **six database queries**:

1. **Tags lookup:**
   ```sql
   SELECT tag_name.name FROM tag_name
   INNER JOIN post_tag ON tag_name.tag_id = post_tag.tag_id
   WHERE post_tag.post_id = post.id AND tag_name IS PRIMARY NAME
   ```
   Only fetches primary tag names (not aliases). Joins through `post_tag` junction table.

2. **Relations lookup:**
   ```sql
   SELECT post_relation.child_id FROM post_relation
   WHERE parent_id = post.id
   ```
   Fetches child post IDs. Relations are bidirectional, so only querying `parent_id` = `post.id` covers all related posts.

3. **Notes lookup:**
   ```sql
   SELECT PostNote.* FROM post_note WHERE post_id = post.id
   ```
   Uses `PostNote::as_select()` for typed loading. Maps results through `Note::new()` to create `Note` objects.

4. **Featured check:**
   ```sql
   SELECT post_feature.post_id FROM post_feature ORDER BY time DESC LIMIT 1
   ```
   Fetches the most recently featured post. Returns `Optional` — if `Some(post.id)`, then `featured = true`.

5. **Custom fields lookup:**
   ```sql
   SELECT custom_field.name, post_custom_field.value, custom_field.color
   FROM post_custom_field
   INNER JOIN custom_field
   WHERE post_custom_field.post_id = post.id
   ```
   Returns tuples of `(name, value, color)` which are mapped to `CustomFieldValue` structs.

6. The `checksum` field is hex-encoded using `hex::encode(&post.checksum)` — the database stores raw bytes but the snapshot stores a hex string.

### Method: `sort_fields(&mut self)`

Sorts four collections for deterministic diffing:
- `tags` — `sort_unstable()` on strings
- `relations` — `sort_unstable()` on i64 IDs
- `notes` — `sort_unstable_by_key(Note::id)` using the note's ID
- `custom_fields` — `sort_unstable_by(|a, b| a.name.cmp(&b.name))` by field name

---

## Public Functions

### `creation_snapshot(conn, client, post_id, post_data) -> ApiResult<()>`

Called when a new post is created. Takes a pre-built `SnapshotData`.

Calls `unary_snapshot` with `ResourceOperation::Created`.

### `feature_snapshot(conn, client, old_featured_post_id, new_featured_post_id) -> QueryResult<()>`

Called when the featured post changes. Handles two cases:

**Early return:** If `old_featured_post_id == Some(new_featured_post_id)`, the feature did not change. Returns `Ok(())` immediately.

**Old feature removal:** If `old_featured_post_id` is `Some(id)`, computes diff between `{"featured": true}` → `{"featured": false}` and inserts a `Modified` snapshot for the old featured post.

**New feature assignment:** Always computes diff between `{"featured": false}` → `{"featured": true}` and inserts a `Modified` snapshot for the new featured post.

This means the post being featured gets a `Modified` snapshot with the `featured` field change, even though no other post fields changed.

### `merge_snapshot(conn, client, absorbed_post_id, merge_to_post_id) -> QueryResult<()>`

Called when one post is merged into another.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Merged`
- `resource_id` = `absorbed_post_id`
- `data` = `[ResourceType::Post, merge_to_post_id]` (JSON array indicating merge target)

### `modification_snapshot(conn, client, post_id, old, new) -> ApiResult<()>`

Called when a post is modified.

1. Calls `sort_fields()` on both `old` and `new` data.
2. Serializes both to JSON via `serde_json::to_value`.
3. Calls `snapshot::value_diff(old_data, new_data)`.
4. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
5. If no diff, inserts nothing.

Uses `ApiResult` — serialization errors become `ApiError`.

### `deletion_snapshot(conn, client, post_id, post_data) -> ApiResult<()>`

Called when a post is deleted.

Calls `unary_snapshot` with `ResourceOperation::Deleted`.

---

## Public Function: `unary_snapshot(conn, client, post_id, post_data, operation) -> ApiResult<()>`

This function is `pub` (accessible outside the module), unlike most other submodule unary_snapshot functions which are private.

1. Calls `sort_fields()` on `post_data`.
2. Serializes data to JSON via `serde_json::to_value`. Maps errors to `ApiError`.
3. Creates and inserts a `NewSnapshot`:
   - `user_id` = `client.id`
   - `operation` = the given operation
   - `resource_type` = `ResourceType::Post`
   - `resource_id` = `post_id`
   - `data` = serialized JSON
4. Maps insert errors to `ApiError`.

---

## Database Queries Summary (retrieve)

| Query | Tables | Conditions | Result |
|---|---|---|---|
| Tags | `tag_name` + `post_tag` | `post_tag.post_id = post.id`, primary name only | `Vec<SmallString>` |
| Relations | `post_relation` | `parent_id = post.id` | `Vec<i64>` (child IDs) |
| Notes | `post_note` | `post_id = post.id` | `Vec<PostNote>` → `Vec<Note>` |
| Featured | `post_feature` | `ORDER BY time DESC LIMIT 1` | `Option<i64>` |
| Custom Fields | `post_custom_field` + `custom_field` | `post_custom_field.post_id = post.id` | `Vec<(name, value, color)>` |

---

## Side Effects

- `retrieve` performs 5 database reads (+ 1 implicit in the Post parameter)
- Each snapshot function inserts 1 row into the `snapshot` table
- `feature_snapshot` may insert up to 2 rows
- `sort_fields` mutates data in-place
- No filesystem operations
