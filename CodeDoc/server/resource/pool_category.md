# Pool Category Resource — `resource/pool_category.rs`

**Source:** `server/src/resource/pool_category.rs` (80 lines)

## Purpose

This file serializes `PoolCategory` model entities into `PoolCategoryInfo` API response structs. Pool categories classify pools into types (e.g. series, collections). Each category has a name, color, usage count, and a default flag.

## Dependencies

- `crate::model::pool_category::PoolCategory` — database model
- `crate::resource::field::Mask` — field selection
- `crate::schema::{pool_category, pool_category_statistics}` — Diesel schema
- `crate::string::SmallString` — compact string type
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM query building
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Field Enum

```rust
pub enum Field {
    Version,
    Name,
    Color,
    Usages,
    Default,
}
```

## Struct

### `PoolCategoryInfo`

The API response shape for a pool category.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `category.last_edit_time` | Resource version |
| `name` | `Option<SmallString>` | `category.name` | Category name |
| `color` | `Option<SmallString>` | `category.color` | Display color |
| `usages` | `Option<i64>` | `pool_category_statistics.usage_count` | How many pools use this category |
| `default` | `Option<bool>` | Computed: `category.id == 0` | True if this is the default category |

**Computed field — `default`:** The default pool category is identified by having ID 0 (the first category in the database). The boolean is computed as `category.id == 0`.

## Constructor Methods

### `PoolCategoryInfo::new(conn, category, fields) -> QueryResult<Self>`

Create a single pool category resource.

**Processing:**
1. Load usage count from `pool_category_statistics` for this category ID
2. Build the struct, applying field mask filters
3. Compute `default` as `category.id == 0`

**No `Batcher`:** This constructor does not use `Batcher` because there is only one related query (usages). The query is unconditional since it is a single lookup.

### `PoolCategoryInfo::all(conn, fields) -> QueryResult<Vec<Self>>`

Load all pool categories.

**Database query:**
```sql
SELECT PoolCategory.*, pool_category_statistics.usage_count
FROM pool_category
INNER JOIN pool_category_statistics ON pool_category.id = pool_category_statistics.pool_category_id
ORDER BY pool_category.id
```

**Processing:** Maps each `(PoolCategory, i64)` tuple to a `PoolCategoryInfo`, applying field mask filters. Returns categories in ID order.
