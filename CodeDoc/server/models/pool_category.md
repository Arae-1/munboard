# Pool Category Model

## Database Tables

- `pool_category` — Stores categories that pools can belong to.

## Structs

### `NewPoolCategory` (Insertable)

Maps to table: `pool_category`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `&'a str` | The name of the pool category. |
| `color` | `&'a str` | A color value for display purposes. |

Diesel annotations:
- `#[diesel(table_name = pool_category)]`
- `#[diesel(check_for_backend(Pg))]`

### `PoolCategory`

Maps to table: `pool_category`

Derives: `Clone`, `AsChangeset`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `name` | `SmallString` | The name of the pool category. |
| `color` | `SmallString` | A color value for display purposes. |
| `last_edit_time` | `DateTime` | Timestamp when the pool category was last edited. |

Diesel annotations:
- `#[diesel(table_name = pool_category)]`
- `#[diesel(check_for_backend(Pg))]`

The `AsChangeset` derive allows using `PoolCategory` as a Diesel changeset for updates.

Custom methods:
- `default() -> SqlLiteral<...>` — Returns a Diesel expression that filters for the default pool category (where `id = 0`). Uses a raw SQL literal for better query planning.

## Type References

- `SmallString` — A bounded string type from `crate::string`.
- `DateTime` — A timestamp type from `crate::time`.
