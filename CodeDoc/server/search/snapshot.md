# snapshot.rs — Snapshot Search

This module provides search for resource snapshots (version history).

## Token Enum

`snapshot.rs:12` — All search tokens for snapshots.

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `User` | `user` | `user.name` |
| `ResourceId` | `id` | `snapshot.resource_id` |

### Enum Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `Operation` | `operation` | `snapshot.operation` | `ResourceOperation` |
| `ResourceType` | `resource-type` | `snapshot.resource_type` | `ResourceType` |

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Time` | `date`, `time` | `snapshot.creation_time` |

## `QueryBuilder` Struct

`snapshot.rs:24` — The snapshot search query builder. Does NOT have a `cache_state`.

### Constructor

`snapshot.rs:82` — The anonymous token is `Token::ResourceType`. Bare words are treated as resource type filters.

### Count

`snapshot.rs:36` — Always runs a filtered count query. There is no optimization path for unfiltered queries.

### Base Query

`snapshot.rs:42` — The base query is:
```sql
SELECT snapshot.id
FROM snapshot
LEFT JOIN user ON ...
```

All filters are applied directly. No TEMP table cache.

## `User` Filter

`snapshot.rs:47` — Filters snapshots by username. Uses `apply_str_filter!` on the LEFT JOINed user.

## `Operation` Filter

`snapshot.rs:48` — Filters snapshots by operation type (`ResourceOperation`). Uses `apply_filter!` with the enum type.

## `ResourceType` Filter

`snapshot.rs:49` — Filters snapshots by resource type (`ResourceType`). Uses `apply_filter!` with the enum type.

## `ResourceId` Filter

`snapshot.rs:50` — Filters snapshots by resource ID. Stored as text. Uses `apply_str_filter!`.

## `Time` Filter

`snapshot.rs:51` — Filters snapshots by creation time. Uses `apply_time_filter!`.

## Sorting

`snapshot.rs:55` — Default sort is `Time` descending. All five tokens are sortable.

## Pagination

`snapshot.rs:73` — Standard OFFSET/LIMIT from `extra_args`.

## Default Token

The anonymous token is `Token::ResourceType`. A bare word is treated as a resource type filter.
