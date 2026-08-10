# Custom Field Resource — `resource/custom_field.rs`

**Source:** `server/src/resource/custom_field.rs` (117 lines)

## Purpose

This file serializes `CustomField` model entities into `CustomFieldInfo` API response structs. It also supplies a bulk-loading function to get custom field values for multiple posts.

## Dependencies

- `crate::model::custom_field::CustomField` — database model
- `crate::resource` — shared helpers (`single`, `order_as`)
- `crate::resource::field::Mask` — field selection
- `crate::schema::{custom_field, post_custom_field}` — Diesel schema
- `crate::string::SmallString` — compact string type
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM query building
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `std::collections::HashMap` — for grouping values by post
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Structs

### `CustomFieldValue`

A simple value struct that represents one custom field assignment on a post.

| Field | Type | Source | Purpose |
|-------|------|--------|---------|
| `name` | `SmallString` | `custom_field.name` | The custom field name |
| `value` | `i64` | `post_custom_field.value` | The assigned value. Must be a non-negative integer (0 is allowed; negative values are rejected). |
| `color` | `SmallString` | `custom_field.color` | Display color for the field |

**Serialization:** All fields are always present (no `Option`). The struct is `Clone`, `Serialize`, `ToSchema`.

### `CustomFieldInfo`

The API response shape for a custom field definition.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `cf.last_edit_time` | Resource version |
| `id` | `Option<i64>` | `cf.id` | Custom field identifier |
| `name` | `Option<SmallString>` | `cf.name` | Field name |
| `description` | `Option<SmallString>` | `cf.description` | Field description |
| `color` | `Option<SmallString>` | `cf.color` | Display color |
| `creation_time` | `Option<DateTime>` | `cf.creation_time` | When created |

**Note:** The `version` field maps from `last_edit_time`, consistent with other resource types.

## Field Enum

```rust
pub enum Field {
    Version,
    Id,
    Name,
    Description,
    Color,
    CreationTime,
}
```

## Constructor Methods

### `CustomFieldInfo::new(conn, custom_field, fields) -> QueryResult<Self>`

Create a single custom field resource. Delegates to `new_batch` and calls `resource::single`.

### `CustomFieldInfo::new_from_id(conn, custom_field_id, fields) -> QueryResult<Self>`

Load from ID, then delegate to `new_batch`.

### `CustomFieldInfo::new_batch(conn, custom_fields, fields) -> QueryResult<Vec<Self>>`

Create multiple custom field resources. The `conn` parameter is unused (prefixed with `_`). No additional database queries are needed because all custom field data comes from the model entities directly.

**Processing:** Maps each `CustomField` model to `CustomFieldInfo`, filtering fields through the mask. This is the simplest resource construction — no `Batcher` is used because there is no related data to load.

### `CustomFieldInfo::new_batch_from_ids(conn, custom_field_ids, fields) -> QueryResult<Vec<Self>>`

Load custom fields by IDs. Uses `resource::order_as` to reorder results. Delegates to `new_batch`.

## Standalone Function

### `get_custom_fields_for_posts(conn, post_ids) -> QueryResult<Vec<Vec<CustomFieldValue>>>`

Bulk-load custom field values for multiple posts. This is called by the `PostInfo` resource when the `CustomFields` field is requested.

**Database query:**
```sql
SELECT post_custom_field.post_id, custom_field.name,
       post_custom_field.value, custom_field.color
FROM post_custom_field
INNER JOIN custom_field ON post_custom_field.custom_field_id = custom_field.id
WHERE post_custom_field.post_id IN (...)
```

**Transform:**
1. Build a `HashMap<i64, Vec<CustomFieldValue>>` keyed by post ID
2. For each post ID in the input slice, return the corresponding values or an empty Vec
3. Result preserves input order via `post_ids.iter().map()`

**Return value:** `Vec<Vec<CustomFieldValue>>` where the outer vector aligns with the input `post_ids` slice.
