# Custom Field Model

## Database Tables

- `custom_field` — Stores named custom fields that can be assigned to posts.

## Structs

### `NewCustomField` (Insertable)

Maps to table: `custom_field`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `&'a str` | The name of the custom field. |
| `description` | `&'a str` | A description of the custom field. |
| `color` | `&'a str` | A color value for display purposes. |

Diesel annotations:
- `#[diesel(table_name = custom_field)]`
- `#[diesel(check_for_backend(Pg))]`

### `CustomField`

Maps to table: `custom_field`

Derives: `Clone`, `PartialEq`, `Eq`, `Debug`, `AsChangeset`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `name` | `SmallString` | The name of the custom field. |
| `description` | `SmallString` | A description of the custom field. |
| `color` | `SmallString` | A color value for display purposes. |
| `creation_time` | `DateTime` | Timestamp when the custom field was created. |
| `last_edit_time` | `DateTime` | Timestamp when the custom field was last edited. |

Diesel annotations:
- `#[diesel(table_name = custom_field)]`
- `#[diesel(check_for_backend(Pg))]`

The `AsChangeset` derive allows the struct to act as a changeset for Diesel update operations.

The `Clone`, `PartialEq`, `Eq`, and `Debug` derives come from standard Rust traits for value comparison and debugging.

## Type References

- `SmallString` — A bounded string type from `crate::string`.
- `DateTime` — A timestamp type from `crate::time`.
