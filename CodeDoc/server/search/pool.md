# pool.rs — Pool Search

This module provides search for pools.

## Token Enum

`pool.rs:15` — All search tokens for pools.

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `CreationTime` | `creation-date`, `creation-time` | `pool.creation_time` |
| `LastEditTime` | `edit-date`, `edit-time`, `last-edit-date`, `last-edit-time` | `pool.last_edit_time` |

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Name` | `name` | `pool_name.name` |
| `Category` | `category` | `pool_category.name` |

### Numeric Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `PostCount` | `post-count` | `pool_statistics.post_count` | `i64` |

## `QueryBuilder` Struct

`pool.rs:32` — The pool search query builder.

### Constructor

`pool.rs:104` — The anonymous token is `Token::Name`. Bare words are treated as pool name filters.

### Count Optimization

`pool.rs:45` — If no filters are set, reads from `database_statistics.pool_count`.

### Base Query

`pool.rs:57` — The base query is:
```sql
SELECT pool.id
FROM pool
INNER JOIN pool_statistics ON ...
INNER JOIN pool_category ON ...
```

## `Name` Filter

`pool.rs:120` — Filters pools by name. Joins `pool_name` table. Uses TEMP tables. Adds `DISTINCT` if multivalued.

## `Category` Filter

`pool.rs:71` — Filters pools by category name. Applied directly on the `pool_category` join from the base query. Does NOT use TEMP tables.

## `PostCount` Filter

`pool.rs:72` — Filters pools by the number of posts they contain.

## Sorting

`pool.rs:77` — Default sort is `CreationTime` descending.

### Primary Name Requirement

`pool.rs:88` — Before sorting, the query filters to only primary pool names:
```rust
.inner_join(pool_name::table).filter(PoolName::primary())
```

### Sortable Tokens

All five tokens are sortable.

## Pagination

`pool.rs:96` — Standard OFFSET/LIMIT from `extra_args`.

## Default Token

The anonymous token for pool search is `Token::Name`. A bare word is treated as a pool name filter.
