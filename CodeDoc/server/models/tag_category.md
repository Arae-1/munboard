# Tag Category Model

## Database Tables

- `tag_category` — Stores categories that tags belong to (e.g. "artist", "character", "copyright", "meta").

## Structs

### `NewTagCategory` (Insertable)

Maps to table: `tag_category`

| Field | Type | Description |
|-------|------|-------------|
| `order` | `i32` | The display order of the category. Lower values appear first. |
| `name` | `&'a str` | The name of the tag category. |
| `color` | `&'a str` | A color value for display purposes. |

Diesel annotations:
- `#[diesel(table_name = tag_category)]`
- `#[diesel(check_for_backend(Pg))]`

### `TagCategory`

Maps to table: `tag_category`

Derives: `Clone`, `PartialEq`, `Eq`, `AsChangeset`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `order` | `i32` | The display order of the category. |
| `name` | `SmallString` | The name of the tag category. |
| `color` | `SmallString` | A color value for display purposes. |
| `last_edit_time` | `DateTime` | Timestamp when the tag category was last edited. |

Diesel annotations:
- `#[diesel(table_name = tag_category)]`
- `#[diesel(check_for_backend(Pg))]`

The `AsChangeset` derive allows using `TagCategory` as a Diesel changeset for updates.

The `PartialEq` and `Eq` derives enable equality comparisons between tag categories.

Custom methods:
- `default() -> SqlLiteral<...>` — Returns a Diesel expression that filters for the default tag category (where `id = 0`). Uses a raw SQL literal for better query planning.

## Type References

- `SmallString` — A bounded string type from `crate::string`.
- `DateTime` — A timestamp type from `crate::time`.
