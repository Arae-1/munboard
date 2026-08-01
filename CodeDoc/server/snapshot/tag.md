# Snapshot — Tag

**File:** `server/src/snapshot/tag.rs` (163 lines)
**Resource Type:** `ResourceType::Tag`

---

## Struct: `SnapshotData`

### Fields

| Field | Type | Description |
|---|---|---|
| `description` | `LargeString` | Text description of the tag |
| `category` | `SmallString` | Name of the tag category |
| `names` | `Vec<SmallString>` | Ordered list of tag names (primary name first, then aliases) |
| `implications` | `Vec<SmallString>` | Primary names of tags this tag implies |
| `suggestions` | `Vec<SmallString>` | Primary names of tags this tag suggests |

Derives: `Clone`, `Serialize`.

### Method: `retrieve(conn, tag) -> QueryResult<Self>`

Takes a `Tag` model and queries the database for related data. Performs **four database queries**:

1. **Category lookup:**
   ```sql
   SELECT tag_category.name FROM tag_category WHERE id = tag.category_id
   ```

2. **Names lookup:**
   ```sql
   SELECT tag_name.name FROM tag_name WHERE tag_id = tag.id
   ```
   Returns all names (primary and aliases) in order.

3. **Implications lookup:**
   ```sql
   SELECT tag_name.name FROM tag_name
   INNER JOIN tag_implication ON tag_name.tag_id = tag_implication.child_id
   WHERE tag_implication.parent_id = tag.id AND tag_name IS PRIMARY
   ```
   Returns the primary names of implied tags (children of this tag's implications). Filters to only primary names.

4. **Suggestions lookup:**
   ```sql
   SELECT tag_name.name FROM tag_name
   INNER JOIN tag_suggestion ON tag_name.tag_id = tag_suggestion.child_id
   WHERE tag_suggestion.parent_id = tag.id AND tag_name IS PRIMARY
   ```
   Returns the primary names of suggested tags (children of this tag's suggestions). Filters to only primary names.

### Method: `sort_fields(&mut self)`

Sorts three collections for deterministic diffing:
- `names` — `sort_unstable()`
- `implications` — `sort_unstable()`
- `suggestions` — `sort_unstable()`

---

## Public Functions

### `creation_snapshot(conn, client, tag_data) -> ApiResult<()>`

Called when a new tag is created. Takes a pre-built `SnapshotData`.

Calls `unary_snapshot` with `ResourceOperation::Created`.

**Note:** Unlike most creation functions, this takes `SnapshotData` directly, not a reference to the model. This is because tags can be created with implications/suggestions already set.

### `new_name_snapshots(conn, client, new_names) -> ApiResult<usize>`

Called when tag auto-creation happens (e.g., when adding a new tag name to a post that does not exist yet). Creates snapshots for multiple new tags at once.

**Parameters:**
- `new_names`: `Vec<SmallString>` — list of new tag names that were auto-created

**Algorithm:**

1. Queries the **default tag category** name:
   ```sql
   SELECT tag_category.name FROM tag_category WHERE tag_category IS DEFAULT
   ```

2. For each name in `new_names`, constructs a `SnapshotData` with:
   - `description` = empty `LargeString`
   - `category` = the default category name
   - `names` = a single-element vector with only this name
   - `implications` = empty vector
   - `suggestions` = empty vector

3. Serializes each into a `NewSnapshot` with:
   - `resource_id` = the first (and only) name from `names`
   - `operation` = `ResourceOperation::Created`

4. **Batch inserts** all snapshots using `insert_into(snapshot::table).execute(conn)` — a single SQL INSERT for efficiency.

5. Returns the number of inserted rows (`usize`).

**Panics** with `TAG_NAME_PANIC_MESSAGE` ("A tag must have at least one name") if any snapshot data has an empty names list. This should never happen since each name populates one `SnapshotData`.

### `merge_snapshot(conn, client, absorbed_tag, merge_to_tag) -> QueryResult<()>`

Called when a tag is merged into another.

Inserts a `NewSnapshot` with:
- `operation` = `ResourceOperation::Merged`
- `resource_id` = `absorbed_tag` (the name string of the absorbed tag)
- `data` = `[ResourceType::Tag, merge_to_tag]` (JSON array indicating merge target)

### `modification_snapshot(conn, client, old, new) -> ApiResult<()>`

Called when a tag is modified.

1. Extracts `resource_id` from `old.names.first()` (the first name acts as the tag's identifier). **Panics** if names is empty.
2. Calls `sort_fields()` on both `old` and `new`.
3. Serializes to JSON via `serde_json::to_value`.
4. Calls `snapshot::value_diff(old_data, new_data)`.
5. If a diff exists, inserts a `Modified` snapshot with the diff as `data`.
6. If no diff, inserts nothing.

### `deletion_snapshot(conn, client, tag_data) -> ApiResult<()>`

Called when a tag is deleted.

Calls `unary_snapshot` with `ResourceOperation::Deleted`.

---

## Private Functions

### `unary_snapshot(conn, client, tag_data, operation) -> ApiResult<()>`

1. Calls `sort_fields()` on `tag_data`.
2. Extracts `resource_id` from the first name. **Panics** if names is empty (with `TAG_NAME_PANIC_MESSAGE`).
3. Serializes to JSON. Maps errors to `ApiError`.
4. Creates and inserts a `NewSnapshot`:
   - `user_id` = `client.id`
   - `operation` = the given operation
   - `resource_type` = `ResourceType::Tag`
   - `resource_id` = first name from the names list
   - `data` = serialized JSON

---

## Constant: `TAG_NAME_PANIC_MESSAGE`

Value: `"A tag must have at least one name"`

Used in panic messages when the names vector is unexpectedly empty during `resource_id` extraction.

---

## Database Queries Summary (retrieve)

| Query | Tables | Conditions | Result |
|---|---|---|---|
| Category | `tag_category` | `id = tag.category_id` | `SmallString` (name) |
| Names | `tag_name` | `tag_id = tag.id` | `Vec<SmallString>` |
| Implications | `tag_name` + `tag_implication` | `tag_implication.parent_id = tag.id`, primary names only | `Vec<SmallString>` |
| Suggestions | `tag_name` + `tag_suggestion` | `tag_suggestion.parent_id = tag.id`, primary names only | `Vec<SmallString>` |

Additional query in `new_name_snapshots`: fetches the default tag category name.

---

## Side Effects

- `retrieve` performs 4 database reads
- Each unary snapshot function inserts 1 row into `snapshot` table
- `new_name_snapshots` performs 1 read + 1 batch INSERT
- `sort_fields` mutates data in-place
- No filesystem operations
