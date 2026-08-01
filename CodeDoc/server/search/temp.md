# temp.rs — Temporary TEMP Tables

This file defines two Diesel table declarations for PostgreSQL temporary tables. These tables are used as a filter cache during complex search queries.

## `matching` Table

`temp.rs:6` — Stores IDs that match positive (non-negated) filters.

```sql
CREATE TEMP TABLE matching (id BIGINT PRIMARY KEY) ON COMMIT DROP
```

| Detail | Value |
|--------|-------|
| Column | `id → Int8` (BIGINT) |
| Primary Key | `id` |
| Scope | TEMP (session-local) |
| Lifetime | Dropped on transaction COMMIT |

### Usage

1. Created on first positive filter by `update_matching_filter_cache!`.
2. First filter: IDs are INSERTed directly.
3. Subsequent positive filters: Intersection — IDs not in the new filter's results are DELETEd.
4. At the end of `build_filtered`, `apply_cache_filters!` adds `WHERE EXISTS (SELECT 1 FROM matching WHERE matching.id = post.id)`.
5. Dropped automatically at transaction end.

## `nonmatching` Table

`temp.rs:12` — Stores IDs that match unnegated versions of negative filters.

```sql
CREATE TEMP TABLE nonmatching (id BIGINT PRIMARY KEY) ON COMMIT DROP
```

| Detail | Value |
|--------|-------|
| Column | `id → Int8` (BIGINT) |
| Primary Key | `id` |
| Scope | TEMP (session-local) |
| Lifetime | Dropped on transaction COMMIT |

### Usage

1. Created on first negative filter by `update_nonmatching_filter_cache!`.
2. First filter: IDs are INSERTed.
3. Subsequent negative filters: Union — IDs are INSERTed with `ON CONFLICT DO NOTHING`.
4. At the end of `build_filtered`, `apply_cache_filters!` adds `WHERE NOT EXISTS (SELECT 1 FROM nonmatching WHERE nonmatching.id = post.id)`.
5. Dropped automatically at transaction end.

## Compatibility Declarations

`temp.rs:18-37` — Uses `diesel::allow_tables_to_appear_in_same_query!` to let the TEMP tables appear in the same query as the following permanent tables:

| TEMP Table | Allowed With |
|------------|-------------|
| `matching` | `pool`, `pool_category`, `pool_statistics`, `post`, `post_favorite`, `post_score`, `post_statistics`, `tag`, `tag_category`, `tag_statistics`, `user` |
| `nonmatching` | `pool`, `pool_category`, `pool_statistics`, `post`, `post_statistics`, `tag`, `tag_category`, `tag_statistics`, `user` |

This is a Diesel requirement. By default, Diesel prevents tables from different sources (schema file vs inline) from appearing in the same query. These declarations override that restriction.

## Why TEMP Tables

Some filters require JOINs to separate tables (tags, pools, favorites, comments). You cannot combine multiple independent JOINs into one SQL query easily. The TEMP table approach lets each complex filter:
1. Run its own subquery.
2. Store the result IDs in a TEMP table.
3. Have the final query filter by TEMP table membership.

This design also allows the negative tag filter optimization. All negative tag filters are unioned into one subquery, then inserted into `nonmatching` in one batch instead of one at a time.
