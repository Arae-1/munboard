# Snapshot Module — Core Framework

**File:** `server/src/snapshot/mod.rs` (197 lines)
**Type:** module root
**Purpose:** Snapshot trait, JSON diff engine, default-toggling snapshots, and submodule declarations.

---

## Submodules Declared

| Submodule | Resource Type |
|---|---|
| `comment` | ResourceType::Comment |
| `custom_field` | ResourceType::CustomField |
| `pool` | ResourceType::Pool |
| `pool_category` | ResourceType::PoolCategory |
| `post` | ResourceType::Post |
| `tag` | ResourceType::Tag |
| `tag_category` | ResourceType::TagCategory |
| `user` | ResourceType::User |
| `user_token` | ResourceType::UserToken |

---

## Struct: `NewSnapshot` (external)

Defined in `crate::model::snapshot`. Fields:

| Field | Type | Description |
|---|---|---|
| `user_id` | `i64` | The ID of the client who triggered the snapshot |
| `operation` | `ResourceOperation` | Created, Modified, Deleted, or Merged |
| `resource_type` | `ResourceType` | The type of resource being snapshotted |
| `resource_id` | `SmallString` | The identifier of the resource (name or ID) |
| `data` | `Value` (serde_json) | The snapshot payload — full JSON for create/delete, diff for modify |

Has an `.insert(conn)` method that writes to the `snapshot` database table.

---

## Public Function: `value_diff(old, new) -> Option<Value>`

**Signature:** `pub fn value_diff(old: Value, new: Value) -> Option<Value>`

The core JSON diff engine. Computes the difference between two `serde_json::Value` instances. Returns `None` if the values are identical.

### Dispatch Logic

1. **Both are `Value::Array`** — calls `array_diff(old_array, new_array)`
2. **Both are `Value::Object`** — calls `object_diff(old_object, new_object)`
3. **Otherwise** — compares by equality. If different, produces a "primitive change" or "property changed type" record.

### Primitive/Type-Change Output Format

When two non-array, non-object values differ, the output is:
```json
{
  "type": "primitive change" | "property changed type",
  "old-value": <old>,
  "new-value": <new>
}
```

The type is `"primitive change"` when both values have the same JSON type (e.g., both strings, both numbers) or when either is null. The type is `"property changed type"` when the types differ (e.g., string vs. number).

A helper `value_index` maps each JSON variant to a numeric index for type comparison:
- Null → 0
- Bool → 1
- Number → 2
- String → 3
- Array → 4
- Object → 5

---

## Private Function: `array_diff(old, new) -> Option<Value>`

**Signature:** `fn array_diff(old: Vec<Value>, new: Vec<Value>) -> Option<Value>`

Computes element-wise diff between two JSON arrays. Returns `None` if no elements were added or removed.

### Algorithm

1. Build a `HashSet` from `old` values for O(1) lookup.
2. Build a boolean vector `new_in_old` — true if each element in `new` existed in `old`.
3. Collect **removed** items: elements in `old` that are not in `new_set`.
4. Collect **added** items: elements in `new` where `new_in_old` is false (using zip).
5. If either `added` or `removed` is non-empty, produce output.

### Output Format
```json
{
  "type": "list change",
  "added": [...],
  "removed": [...]
}
```

### Key Behavior
- Uses value equality via `HashSet` — two identical JSON values at different positions count as "not changed."
- **Does not track reordering.** If elements are shuffled but the set is identical, diff returns `None`.
- Duplicate handling: only the first occurrence of a value in `old` is matched. If `new` has two identical items and `old` had one, one will appear as "added."

---

## Private Function: `object_diff(old, new) -> Option<Value>`

**Signature:** `fn object_diff(old: Map<String, Value>, mut new: Map<String, Value>) -> Option<Value>`

Computes diff between two JSON key/value maps. Returns `None` if no keys were added, removed, or changed.

### Algorithm

1. **Identify added keys** — keys in `new` that are not in `old`. Remove them from the `new` map to avoid reprocessing.
2. **Iterate over `old` keys**:
   - If the key is absent from `new`: record as **"deleted property"** with the old value.
   - If the key is present: call `value_diff(old_value, new_value)` recursively. If there is a change, record it under the key.
3. **Process added keys** — for each key that was only in `new`, record as **"added_property"** with the new value.
4. If the diff map is non-empty, wrap it in an `"object change"` envelope.

### Output Format
```json
{
  "type": "object change",
  "value": {
    "changed_key": { <sub-diff> },
    "deleted_key": {
      "type": "deleted property",
      "value": <old_value>
    },
    "new_key": {
      "type": "added_property",
      "value": <new_value>
    }
  }
}
```

### Key Behavior
- **Recursive** — nested objects and arrays are diffed by recursive calls to `value_diff`.
- **Modifies `new` in-place** — added keys are removed from the mutable `new` map to prevent double-processing.

---

## Public Function: `set_default_snapshot(conn, client, old_default_name, new_default_name, resource_type)`

**Signature:** `fn set_default_snapshot(conn: &mut PgConnection, client: Client, old_default_name: SmallString, new_default_name: SmallString, resource_type: ResourceType) -> QueryResult<()>`

Creates two snapshots for a "set default" operation on a category (pool_category or tag_category).

### Code Paths

1. **Early return**: If `old_default_name == new_default_name`, no snapshots are created (no change).
2. **Old default snapshot**: Computes diff between `{"default": true}` → `{"default": false}` and inserts a `Modified` snapshot with `resource_id = old_default_name`.
3. **New default snapshot**: Computes diff between `{"default": false}` → `{"default": true}` and inserts a `Modified` snapshot with `resource_id = new_default_name`.

### Panic
Uses `.expect("There must be a diff")` on the output of `value_diff`. These calls should always produce a diff because the input values differ.

---

## Tests

### `json_diff`

Verifies that `value_diff` correctly diffs a complex object with:
- A null-to-string change (`source: null` → `"new source"`)
- An empty-array-to-populated-array change (`notes: []` → array of note objects)

The test asserts exact structural equality against an expected diff output, confirming the recursive nesting behavior.

### Configuration
- `POST_FILE_DIR`: directory path for post file storage

---

## Design Pattern

Every submodule follows this pattern:
1. Define a `SnapshotData` struct (or inline JSON) with the tracked fields.
2. Provide `creation_snapshot` / `modification_snapshot` / `deletion_snapshot` public functions.
3. For modification: compute a diff between old and new data; only insert a snapshot if there is a diff.
4. For creation/deletion: insert the full data directly (a "unary" snapshot).
5. Some submodules add specialized snapshots: `merge_snapshot`, `feature_snapshot`, `set_default_snapshot`, `new_name_snapshots`.
