# Tag Model

## Database Tables

- `tag` — Stores tag entities.
- `tag_name` — Stores names for tags (supports multiple names with ordering, aliases).
- `tag_implication` — Stores implication relationships between tags (if parent tag is applied, child tag is implied).
- `tag_suggestion` — Stores suggestion relationships between tags (if parent tag is applied, child tag is suggested).

## Structs

### `NewTag` (Insertable)

Maps to table: `tag`

Derives: `Clone`, `Copy`, `Default`

| Field | Type | Description |
|-------|------|-------------|
| `category_id` | `i64` | Foreign key to `tag_category.id`. |
| `description` | `&'a str` | A description of the tag. |

Diesel annotations:
- `#[diesel(table_name = tag)]`
- `#[diesel(check_for_backend(Pg))]`

The `Default` derive produces a `NewTag` with `category_id = 0` and empty `description`.

### `Tag`

Maps to table: `tag`

Derives: `Clone`, `AsChangeset`, `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `category_id` | `i64` | Foreign key to `tag_category.id`. |
| `description` | `LargeString` | A description of the tag. |
| `creation_time` | `DateTime` | Timestamp when the tag was created. |
| `last_edit_time` | `DateTime` | Timestamp when the tag was last edited. |

Diesel annotations:
- `#[diesel(table_name = tag)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(TagCategory, foreign_key = category_id))]`

Relationships:
- Belongs to `TagCategory` via `category_id`.

The `AsChangeset` derive allows using `Tag` as a Diesel changeset for updates.

### `NewTagName` (Insertable)

Maps to table: `tag_name`

| Field | Type | Description |
|-------|------|-------------|
| `tag_id` | `i64` | Foreign key to `tag.id`. |
| `order` | `i32` | The ordering index of this name. Index 0 is the primary name. |
| `name` | `&'a str` | A name for the tag. |

Diesel annotations:
- `#[diesel(table_name = tag_name)]`
- `#[diesel(check_for_backend(Pg))]`

### `TagName`

Maps to table: `tag_name`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `tag_id` | `i64` | Part of composite primary key. Foreign key to `tag.id`. |
| `order` | `i32` | Part of composite primary key. The ordering index. |
| `name` | `SmallString` | A name for the tag. |

Diesel annotations:
- `#[diesel(table_name = tag_name)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(tag_id, order))]`
- `#[diesel(belongs_to(Tag))]`

Relationships:
- Belongs to `Tag` via `tag_id`.

Composite primary key: `(tag_id, order)`.

Custom methods:
- `primary() -> SqlLiteral<...>` — Returns a Diesel expression that filters rows to only the primary name (where `order = 0`). Uses a raw SQL literal instead of `order.eq(0)` for better query performance (up to 15x faster). PostgreSQL can optimize queries better when the exact value is known at query planning time.

### `TagImplication`

Maps to table: `tag_implication`

Derives: `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `parent_id` | `i64` | Part of composite primary key. Foreign key to `tag.id` (the implying tag). |
| `child_id` | `i64` | Part of composite primary key. Foreign key to `tag.id` (the implied tag). |

Diesel annotations:
- `#[diesel(table_name = tag_implication)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(parent_id, child_id))]`
- `#[diesel(belongs_to(Tag, foreign_key = parent_id))]`

This struct implements `Insertable` directly.

Relationships:
- Belongs to `Tag` via `parent_id`.
- Explicit joinable declaration: `diesel::joinable!(tag_implication -> tag (parent_id))`.

Composite primary key: `(parent_id, child_id)`.

### `TagSuggestion`

Maps to table: `tag_suggestion`

Derives: `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `parent_id` | `i64` | Part of composite primary key. Foreign key to `tag.id` (the parent tag). |
| `child_id` | `i64` | Part of composite primary key. Foreign key to `tag.id` (the suggested tag). |

Diesel annotations:
- `#[diesel(table_name = tag_suggestion)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(parent_id, child_id))]`
- `#[diesel(belongs_to(Tag, foreign_key = parent_id))]`

This struct implements `Insertable` directly.

Relationships:
- Belongs to `Tag` via `parent_id`.
- Explicit joinable declaration: `diesel::joinable!(tag_suggestion -> tag (parent_id))`.

Composite primary key: `(parent_id, child_id)`.

## Explicit Joinable Declarations

- `diesel::joinable!(tag_implication -> tag (parent_id))` — Allows Diesel to join `tag_implication` to `tag` using the `parent_id` foreign key.
- `diesel::joinable!(tag_suggestion -> tag (parent_id))` — Allows Diesel to join `tag_suggestion` to `tag` using the `parent_id` foreign key.

## Type References

- `DateTime` — A timestamp type from `crate::time`.
- `LargeString` — A string type from `crate::string`.
- `SmallString` — A bounded string type from `crate::string`.
