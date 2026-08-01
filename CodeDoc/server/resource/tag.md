# Tag Resource — `resource/tag.rs`

**Source:** `server/src/resource/tag.rs` (239 lines)

## Purpose

This file serializes `Tag` model entities into `TagInfo` API response structs. Tags are metadata labels attached to posts. This file also defines `MicroTag`, a lightweight tag representation used by the post resource, pool resource, and other tag relations.

## Dependencies

- `crate::model::tag::{Tag, TagImplication, TagName, TagSuggestion}` — database models
- `crate::resource` — shared helpers (`single`, `order_as`, `order_transformed_as`, `collect_names`)
- `crate::resource::field::{Batcher, Mask}` — field selection
- `crate::schema::{tag, tag_category, tag_implication, tag_name, tag_statistics, tag_suggestion}` — Diesel schema
- `crate::string::{LargeString, SmallString}` — string types
- `crate::time::DateTime` — timestamp type
- `diesel` — ORM including `GroupedBy`, `BelongingToDsl`, `JoinOnDsl`
- `serde::Serialize` — serialization
- `serde_with::skip_serializing_none` — omit `None` fields
- `server_macros::non_nullable_options` — derive helper
- `std::collections::{HashMap, HashSet}` — data structures
- `std::sync::Arc` — shared ownership
- `strum::EnumString` — string-to-enum parsing
- `utoipa::ToSchema` — OpenAPI schema generation

## Structs

### `MicroTag`

A lightweight tag representation. Used when the full tag resource is not needed — in post tags, implications, suggestions, and pool contexts.

| Field | Type | Source | Purpose |
|-------|------|--------|---------|
| `names` | `Arc<[SmallString]>` | `tag_name` table | List of tag names (aliases). The first name is the primary name. |
| `category` | `SmallString` | `tag_category.name` | Category name this tag belongs to |
| `usages` | `i64` | `tag_statistics.usage_count` | How many posts use this tag |

**Serialization:** All fields are always present. No field-level selection. The `Arc<[SmallString]>` allows shared ownership of name lists — the same `Arc` can be referenced by multiple `MicroTag` instances.

### `TagInfo`

The full API response shape for a tag.

| Field | Type | Model Source | Notes |
|-------|------|-------------|-------|
| `version` | `Option<DateTime>` | `tag.last_edit_time` | Resource version |
| `description` | `Option<LargeString>` | `tag.description` | Markdown description |
| `creation_time` | `Option<DateTime>` | `tag.creation_time` | When created |
| `last_edit_time` | `Option<DateTime>` | `tag.last_edit_time` | When last edited |
| `category` | `Option<SmallString>` | Joined from `tag_category` | Category name |
| `names` | `Option<Vec<SmallString>>` | From `tag_name` grouped by tag | Ordered list of aliases |
| `implications` | `Option<Vec<MicroTag>>` | From `tag_implication` joined with tag data | Implied tags (auto-added on use) |
| `suggestions` | `Option<Vec<MicroTag>>` | From `tag_suggestion` joined with tag data | Suggested tags (shown to user on use) |
| `usages` | `Option<i64>` | `tag_statistics.usage_count` | Post usage count |

## Field Enum

```rust
pub enum Field {
    Version, Description, CreationTime, LastEditTime,
    Category, Names, Implications, Suggestions, Usages,
}
```

## Constructor Methods

### `TagInfo::new(conn, tag, fields) -> QueryResult<Self>`

Create a single tag resource. Delegates to `new_batch` and calls `resource::single`.

### `TagInfo::new_from_id(conn, tag_id, fields) -> QueryResult<Self>`

Load from ID, then delegate to `new_batch`.

### `TagInfo::new_batch(conn, tags, fields) -> QueryResult<Vec<Self>>`

Create multiple tag resources. Uses `Batcher` for conditional loading.

**Conditionally loaded data:**

| Field | Helper Function | Data Source |
|-------|----------------|-------------|
| `Category` | `get_categories` | `tag` INNER JOIN `tag_category` |
| `Names` | `get_names` | `TagName::belonging_to(tags)` grouped by tag |
| `Implications` | `get_implications` | Multi-table join: tag_implication, tag, tag_statistics, tag_name |
| `Suggestions` | `get_suggestions` | Multi-table join: tag_suggestion, tag, tag_statistics, tag_name |
| `Usages` | `get_usages` | `tag_statistics` |

**Processing:** Reverse-iteration and pop pattern.

### `TagInfo::new_batch_from_ids(conn, tag_ids, fields) -> QueryResult<Vec<Self>>`

Load tags by IDs. Uses `resource::order_as`. Delegates to `new_batch`.

## Helper Functions

### `get_categories(conn, tags) -> QueryResult<Vec<SmallString>>`

Load category names. Inner-joins `tag` with `tag_category`. Uses `resource::order_transformed_as` to align with input order.

### `get_names(conn, tags) -> QueryResult<Vec<Vec<SmallString>>>`

Load tag names. Uses `TagName::belonging_to(tags)` ordered by `tag_name::order`. Groups by tag using `GroupedBy`. Each group becomes a `Vec<SmallString>`.

### `get_implications(conn, tags) -> QueryResult<Vec<Vec<MicroTag>>>`

Load implied tags. Implied tags are automatically added when a user adds this tag to a post.

**Step 1 — Load implication records:**
```sql
SELECT TagImplication.*, tag.category_id, tag_statistics.usage_count
FROM tag_implication
INNER JOIN tag ON tag.id = tag_implication.child_id
INNER JOIN tag_statistics ON tag.id = tag_statistics.tag_id
INNER JOIN tag_name ON tag_name.tag_id = tag.id
WHERE tag_implication.tag_id IN (...)
  AND tag_name.primary = true
ORDER BY tag_name.name
```

Filters to primary names. Orders by tag name alphabetically.

**Step 2 — Load all names for implication targets:**
Collects child tag IDs. Queries `tag_name` for all names. Uses `resource::collect_names` to build the name map.

**Step 3 — Load all category names:**
Queries `tag_category` to build a `HashMap<i64, SmallString>`.

**Step 4 — Build MicroTags:**
Groups implications by parent tag. Constructs `MicroTag` for each implication target.

### `get_suggestions(conn, tags) -> QueryResult<Vec<Vec<MicroTag>>>`

Load suggested tags. The structure is identical to `get_implications` but uses `tag_suggestion` instead of `tag_implication`. Suggested tags are shown to users as recommendations when they add this tag.

**Database query pattern mirrors implications exactly**, substituting `tag_suggestion` for `tag_implication`. Steps 2-4 are duplicated code — they load a separate name map and category map for suggestions even though the data is identical to the implications path.

### `get_usages(conn, tags) -> QueryResult<Vec<i64>>`

Load usage counts from `tag_statistics`. Uses `resource::order_transformed_as` to align with input order.
