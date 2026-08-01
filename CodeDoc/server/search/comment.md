# comment.rs — Comment Search

This module provides search for comments.

## Token Enum

`comment.rs:11` — All search tokens for comments.

### Numeric Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `Id` | `id` | `comment.id` | `i64` |
| `Post` | `post` | `comment.post_id` | `i64` |
| `Score` | `score` | `comment_statistics.score` | `i64` |

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Text` | `text` | `comment.text` |
| `User` | `user`, `author` | `user.name` |

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `CreationTime` | `creation-date`, `creation-time` | `comment.creation_time` |
| `LastEditTime` | `edit-date`, `edit-time`, `last-edit-date`, `last-edit-time` | `comment.last_edit_time` |

## `QueryBuilder` Struct

`comment.rs:31` — The comment search query builder. Does NOT have a `cache_state` (no TEMP tables needed).

### Constructor

`comment.rs:111` — The anonymous token is `Token::Text`. Bare words are treated as comment text filters.

### Count Optimization

`comment.rs:44` — If no filters AND no user preferences, reads from `database_statistics.comment_count`.

### Base Query

`comment.rs:56` — The base query is:
```sql
SELECT comment.id
FROM comment
INNER JOIN comment_statistics ON ...
LEFT JOIN user ON ...
```

All filters are applied directly to this base query. No TEMP table cache is used because all filters reference columns in the base query's tables (or the LEFT JOINed user table).

## `Text` Filter

`comment.rs:68` — Filters comments by text content. Uses `apply_str_filter!` directly.

## `User` Filter

`comment.rs:71` — Filters comments by username. Uses the `LEFT JOIN user` from the base query.

## `Post` Filter

`comment.rs:67` — Filters comments by post ID.

## `Score` Filter

`comment.rs:72` — Filters comments by score from `comment_statistics`.

## User Preferences

`comment.rs:76` — After all filters are applied, posts hidden by user preferences are excluded with a `NOT EXISTS` subquery on `comment.post_id`. This prevents comments on hidden posts from appearing.

## Sorting

`comment.rs:82` — Default sort is `CreationTime` descending. All seven tokens are sortable.

## Pagination

`comment.rs:102` — Standard OFFSET/LIMIT from `extra_args`.

## Default Token

The anonymous token for comment search is `Token::Text`. A bare word is treated as a comment text filter.
