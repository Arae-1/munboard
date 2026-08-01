# user.rs — User Search

This module provides search for users.

## Token Enum

`user.rs:11` — All search tokens for users.

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Name` | `name` | `user.name` |

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `CreationTime` | `creation-date`, `creation-time` | `user.creation_time` |
| `LastLoginTime` | `login-date`, `login-time`, `last-login-date`, `last-login-time` | `user.last_login_time` |

## `QueryBuilder` Struct

`user.rs:26` — The user search query builder. Does NOT have a `cache_state`.

### Constructor

`user.rs:87` — The anonymous token is `Token::Name`. Bare words are treated as username filters.

### Count Optimization

`user.rs:39` — If no filters, reads from `database_statistics.user_count`.

### Base Query

`user.rs:51` — The base query is:
```sql
SELECT user.id
FROM user
```

This is the simplest base query. Only one table.

## `Name` Filter

`user.rs:56` — Filters users by name. Uses `apply_str_filter!` directly.

## `CreationTime` Filter

`user.rs:57` — Filters users by account creation time.

## `LastLoginTime` Filter

`user.rs:58` — Filters users by last login time.

## Sorting

`user.rs:62` — Default sort is `Name` descending. All three tokens are sortable.

## Pagination

`user.rs:78` — Standard OFFSET/LIMIT from `extra_args`.

## Default Token

The anonymous token is `Token::Name`. A bare word is treated as a username filter.
