# Pool Model

## Database Tables

- `pool` — Stores pools (collections of posts with an order).
- `pool_name` — Stores names for pools (supports multiple names with ordering).
- `pool_post` — Junction table linking posts to pools with an order position.

## Structs

### `NewPool` (Insertable)

Maps to table: `pool`

| Field | Type | Description |
|-------|------|-------------|
| `category_id` | `i64` | Foreign key to `pool_category.id`. |
| `description` | `&'a str` | A description of the pool. |
| `launch_date` | `Option<DateTime>` | An optional date for the pool. Null if not set. |

Diesel annotations:
- `#[diesel(table_name = pool)]`
- `#[diesel(check_for_backend(Pg))]`

### `Pool`

Maps to table: `pool`

Derives: `Clone`, `AsChangeset`, `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Primary key. Auto-increment identifier. |
| `category_id` | `i64` | Foreign key to `pool_category.id`. |
| `description` | `LargeString` | A description of the pool. |
| `creation_time` | `DateTime` | Timestamp when the pool was created. |
| `last_edit_time` | `DateTime` | Timestamp when the pool was last edited. |
| `launch_date` | `Option<DateTime>` | An optional launch date. |

Diesel annotations:
- `#[diesel(table_name = pool)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(belongs_to(PoolCategory, foreign_key = category_id))]`

Relationships:
- Belongs to `PoolCategory` via `category_id`.

The `AsChangeset` derive allows using `Pool` as a Diesel changeset for updates.

### `NewPoolName` (Insertable)

Maps to table: `pool_name`

| Field | Type | Description |
|-------|------|-------------|
| `pool_id` | `i64` | Foreign key to `pool.id`. |
| `order` | `i32` | The ordering index of this name. Index 0 is the primary name. |
| `name` | `&'a str` | A name for the pool. |

Diesel annotations:
- `#[diesel(table_name = pool_name)]`
- `#[diesel(check_for_backend(Pg))]`

### `PoolName`

Maps to table: `pool_name`

Derives: `Associations`, `Identifiable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `pool_id` | `i64` | Part of composite primary key. Foreign key to `pool.id`. |
| `order` | `i32` | Part of composite primary key. The ordering index. |
| `name` | `SmallString` | A name for the pool. |

Diesel annotations:
- `#[diesel(table_name = pool_name)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(pool_id, order))]`
- `#[diesel(belongs_to(Pool))]`

Relationships:
- Belongs to `Pool` via `pool_id`.

Composite primary key: `(pool_id, order)`.

Custom methods:
- `primary() -> SqlLiteral<...>` — Returns a Diesel expression that filters rows to only the primary name (where `order = 0`). Uses a raw SQL literal instead of `order.eq(0)` for better query performance (up to 15x faster) because PostgreSQL can optimize queries better when the exact value is known at query planning time.

### `PoolPost`

Maps to table: `pool_post`

Derives: `Associations`, `Identifiable`, `Insertable`, `Queryable`, `Selectable`

| Field | Type | Description |
|-------|------|-------------|
| `pool_id` | `i64` | Part of composite primary key. Foreign key to `pool.id`. |
| `post_id` | `i64` | Part of composite primary key. Foreign key to `post.id`. |
| `order` | `i64` | The position of the post within the pool. |

Diesel annotations:
- `#[diesel(table_name = pool_post)]`
- `#[diesel(check_for_backend(Pg))]`
- `#[diesel(primary_key(pool_id, post_id))]`
- `#[diesel(belongs_to(Pool), belongs_to(Post))]`

Relationships:
- Belongs to `Pool` via `pool_id`.
- Belongs to `Post` via `post_id`.

Composite primary key: `(pool_id, post_id)`.

This struct implements `Insertable` directly (no separate `New*` struct) because all its fields are required on insert.

## Type References

- `DateTime` — A timestamp type from `crate::time`.
- `LargeString` — An unbounded or large string type from `crate::string`.
- `SmallString` — A bounded string type from `crate::string`.
