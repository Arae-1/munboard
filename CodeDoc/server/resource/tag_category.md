# Tag Category Resource — `resource/tag_category.rs`

**Source:** `server/src/resource/tag_category.rs` (93 lines)

## Purpose

This file serializes `TagCategory` model entities into `TagCategoryInfo` API response structs. Tag categories classify tags into types (e.g. character, artist, media type). Each category has a name, color, display order, usage count, and a default flag.

## Dependencies

- `crate::app::Context` — application context (config, client)
- `crate::model::enums::UserRank` — user rank enum (for anonymous user filtering)
- `crate::model::tag_category::TagCategory` — database model
- `crate::resource::field::Mask` — field selection
- `crate::schema::{tag_category, tag_category_statistics}` — Diesel schema
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
    Order,
    Default,
}
```

## Struct

### `TagCategoryInfo`

The API response shape for a tag category.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `category.last_edit_time` | Resource version |
| `name` | `Option<SmallString>` | `category.name` | Category name |
| `color` | `Option<SmallString>` | `category.color` | Display color |
| `usages` | `Option<i64>` | `tag_category_statistics.usage_count` | How many tags use this category |
| `order` | `Option<i32>` | `category.order` | Display order (ascending) |
| `default` | `Option<bool>` | Computed: `category.id == 0` | True if this is the default category |

**Computed field — `default`:** The default tag category has ID 0. The boolean is computed as `category.id == 0`.

## Constructor Methods

### `TagCategoryInfo::new(conn, category, fields) -> QueryResult<Self>`

Create a single tag category resource.

**Processing:**
1. Load usage count from `tag_category_statistics` for this category ID
2. Build the struct, applying field mask filters
3. Compute `default` as `category.id == 0`

No `Batcher` is used — the single usage count query is unconditional.

### `TagCategoryInfo::all(conn, ctx, fields) -> QueryResult<Vec<Self>>`

Load all tag categories.

**Database query:**
```sql
SELECT TagCategory.*, tag_category_statistics.usage_count
FROM tag_category
INNER JOIN tag_category_statistics ON ...
ORDER BY tag_category.order
```

**Anonymous user filtering:** When the client is anonymous (`UserRank::Anonymous`), blacklisted categories are excluded:
```rust
tag_categories = tag_categories.filter(
    tag_category::name.ne_all(&ctx.config.anonymous_preferences.tag_category_blacklist)
);
```

**Processing:** Maps each `(TagCategory, i64)` tuple to a `TagCategoryInfo`. Returns categories in display order.
