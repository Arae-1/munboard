# Snapshot Resource — `resource/snapshot.rs`

**Source:** `server/src/resource/snapshot.rs` (302 lines)

## Purpose

This file serializes `Snapshot` model entities into `SnapshotInfo` API response structs. Snapshots record the history of resource changes — who changed what, what operation happened, and what the data looked like before/after. The data field contains full resource data for creation/deletion events and recursive diffs for modification events.

The file contains extensive doc comments (200+ lines) that describe the JSON structure of snapshot data for each resource type and operation. These serve as the primary API documentation for snapshot consumers.

## Dependencies

- `crate::config::Config` — application configuration
- `crate::model::enums::{AvatarStyle, ResourceOperation, ResourceType}` — enum types
- `crate::model::snapshot::Snapshot` — database model
- `crate::resource` — shared helpers (`order_as`, `order_like`)
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::resource::user::MicroUser` — lightweight user representation
- `crate::schema::{snapshot, user}` — Diesel schema
- `crate::string::SmallString` — compact string type
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM query building
- `serde::Serialize` — serialization
- `serde_json::Value` — generic JSON value for the data field
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Field Enum

```rust
pub enum Field {
    User,
    Operation,
    Type,
    Id,
    Data,
    Time,
}
```

## Struct

### `SnapshotInfo`

The API response shape for a snapshot.

| Field | Type | Model Source | Serialized As | Notes |
|-------|------|-------------|---------------|-------|
| `user` | `Option<Option<MicroUser>>` | Joined from `user` table | `user` | Who made the change |
| `operation` | `Option<ResourceOperation>` | `snapshot.operation` | `operation` | Created/Modified/Deleted/Merged |
| `resource_type` | `Option<ResourceType>` | `snapshot.resource_type` | `type` | Renamed via `#[serde(rename)]` |
| `resource_id` | `Option<SmallString>` | `snapshot.resource_id` | `id` | Renamed via `#[serde(rename)]` |
| `data` | `Option<Value>` | `snapshot.data` | `data` | JSON data (full or diff) |
| `time` | `Option<DateTime>` | `snapshot.creation_time` | `time` | When the snapshot was created |

**Serialization renames:**
- `resource_type` is serialized as `"type"` in JSON
- `resource_id` is serialized as `"id"` in JSON

## Constructor Methods

### `SnapshotInfo::new_batch(conn, config, snapshots, fields) -> QueryResult<Vec<Self>>`

Create multiple snapshot resources. Uses `Batcher` for conditional loading of users only. All other fields come directly from the snapshot model.

**Conditionally loaded data:**

| Field | Helper Function | Data Source |
|-------|----------------|-------------|
| `User` | `get_users` | `snapshot` INNER JOIN `user` |

**Processing:** Reverse-iteration and pop pattern. The `data` field is serialized as a raw `serde_json::Value`, preserving whatever JSON structure the snapshot contains.

### `SnapshotInfo::new_batch_from_ids(conn, config, snapshot_ids, fields) -> QueryResult<Vec<Self>>`

Load snapshots by IDs. Uses `resource::order_as`. Delegates to `new_batch`.

## Helper Functions

### `get_users(conn, config, snapshots) -> QueryResult<Vec<Option<MicroUser>>>`

Load the users who created the snapshots. Inner-joins `snapshot` with `user`. Uses `resource::order_like` to align results with input order. Missing users become `None`.

**Note:** Unlike other resource types, there is no `new` or `new_from_id` method. Snapshots are only constructed in batches.

## Snapshot Data Structure (from doc comments)

### ResourceOperation values

| Value | Meaning |
|-------|---------|
| `"created"` | The resource was created |
| `"modified"` | The resource was modified |
| `"deleted"` | The resource was deleted |
| `"merged"` | The resource was merged into another |

### Resource ID mapping by ResourceType

| ResourceType | Resource ID format |
|-------------|-------------------|
| `"tag"` | First tag name at the time |
| `"tag_category"` | Tag category name at the time |
| `"post"` | Post ID |
| `"pool"` | Pool ID |
| `"pool_category"` | Pool category name at the time |

### Data field by operation

**Creation/deletion:** The data field contains the full resource data as a JSON object. Structure varies by resource type.

**Modification:** The data field contains a recursive property-wise diff. The diff uses a structured format with type markers:

| Diff Type | Keys | Meaning |
|-----------|------|---------|
| `"object change"` | `value` | Dictionary of property changes |
| `"primitive change"` | `old-value`, `new-value` | A single value changed |
| `"list change"` | `removed`, `added` | Elements removed/added from a list |
| `"added property"` | `value` | A new property was added |
| `"deleted property"` | `value` | A property was deleted |

**Merge:** The data field is a tuple `[resource_type, resource_id]` identifying the target entity.
