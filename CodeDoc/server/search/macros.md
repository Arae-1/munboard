# macros.rs — Search Macros

This file contains Rust macros for dynamic Diesel query building. Diesel does not support generic functions well. These macros solve that problem.

## `apply_filter!`

`macros.rs:6` — Applies a non-string, non-time filter to a query.

```
apply_filter!($query, $expression, $filter, $condition_type)
```

**Logic:**
1. Parses `$filter.condition` using `parse::condition::<$condition_type>()` to get a `Condition`.
2. Calls `apply_condition!` with the parsed condition.
3. Returns `ApiResult<BoxedQuery>`.

Used for numeric columns, enum columns, and other non-text, non-time columns.

## `apply_time_filter!`

`macros.rs:14` — Applies a time-based filter.

```
apply_time_filter!($query, $expression, $filter)
```

**Logic:**
1. Parses `$filter.condition` using `parse::time_condition()` to get `Condition<Range<DateTime>>`.
2. If the condition is `Condition::Values(times)`: Builds a `tstzmultirange` SQL expression using `@>` (contains) operator. Negation wraps with `NOT`.
3. If `Condition::GreaterEq(time)`: Converts to `Condition::GreaterEq(time.start)`.
4. If `Condition::LessEq(time)`: Converts to `Condition::LessEq(time.start)`.
5. If `Condition::Range(range)`: Converts to `Condition::Range(range.start.start..range.end.end)`.
6. For non-Values variants, calls `apply_condition!`.

**Why this design:** Time values are always implicit ranges. A single date like `2024` is parsed as `2024-01-01 .. 2025-01-01`. When used as an equality filter, a `tstzmultirange` is built to contain-check the column. When used as greater/less/range, the range's start/end points are used.

## `apply_str_filter!`

`macros.rs:42` — Applies a string/text filter.

```
apply_str_filter!($query, $expression, $filter)
```

**Logic:**
1. Parses `$filter.condition` using `parse::str_condition()` to get `StrCondition`.
2. **Regular condition** → Calls `apply_condition!` with the parsed condition.
3. **WildCard condition** → Uses `lower($expression).like(pattern)` or `lower($expression).not_like(pattern)` depending on negation. The `lower()` call forces PostgreSQL to use TEXT indexes instead of CITEXT indexes (CITEXT indexes do not work with LIKE patterns).

## `apply_distinct_if_multivalued!`

`macros.rs:64` — Adds `DISTINCT` to a query if the filter has multiple values.

```
apply_distinct_if_multivalued!($query, $filter)
```

**Logic:** Calls `parse::is_multivalued(filter.condition)`. Returns true if the condition contains a comma, a `*` wildcard, or a `..` range. This optimization prevents duplicate rows when joining to a table that could have multiple matching rows.

## `update_matching_filter_cache!`

`macros.rs:75` — Inserts or intersects matching IDs into the TEMP table.

```
update_matching_filter_cache!($conn, $filtered_ids, $column, $state)
```

**Logic:**
1. If `state.completed` is true, does nothing (cache is locked).
2. If `state.has_matching` is false: Creates the `matching` TEMP table and inserts all `$filtered_ids` into it. Sets `has_matching = true`.
3. If `state.has_matching` is true: Deletes rows from `matching` where the ID is not in the `$filtered_ids` (intersection).
4. Returns `diesel::QueryResult<usize>`.

## `update_nonmatching_filter_cache!`

`macros.rs:106` — Unions nonmatching IDs into the TEMP table.

```
update_nonmatching_filter_cache!($conn, $filtered_ids, $state)
```

**Logic:**
1. If `state.completed` is true, does nothing.
2. If `state.has_nonmatching` is false: Creates the `nonmatching` TEMP table and inserts all `$filtered_ids`. Sets `has_nonmatching = true`.
3. If `state.has_nonmatching` is true: Inserts with `ON CONFLICT DO NOTHING` (union without duplicates).
4. Returns `diesel::QueryResult<usize>`.

## `update_filter_cache!`

`macros.rs:138` — Routes a filter's results to the correct TEMP table.

```
update_filter_cache!($conn, $filtered_ids, $column, $filter, $state)
```

**Logic:**
- If `filter.negated` is true → Calls `update_nonmatching_filter_cache!`.
- If `filter.negated` is false → Calls `update_matching_filter_cache!`.

## `apply_cache_filters!`

`macros.rs:152` — Applies the cached TEMP table results to the main query.

```
apply_cache_filters!($query, $column, $state)
```

**Logic:**
1. Sets `state.completed = true` (locks future cache writes).
2. If `state.has_matching` is true: Adds a WHERE EXISTS subquery on the `matching` table.
3. If `state.has_nonmatching` is true: Adds a WHERE NOT EXISTS subquery on the `nonmatching` table.
4. Returns the filtered query.

## `apply_sort!`

`macros.rs:177` — Applies a sort to a query.

```
apply_sort!($query, $expression, $sort)
```

**Logic:**
- `Order::Asc` → `.then_order_by($expression.asc())`
- `Order::Desc` → `.then_order_by($expression.desc().nulls_last())`

Uses `then_order_by` (not `order_by`) so multiple sorts can be chained.

## `apply_random_sort!`

`macros.rs:189` — Applies seeded random ordering.

```
apply_random_sort!($conn, $client, $query, $criteria)
```

**Logic:**
1. Calls `set_seed()` to set PostgreSQL's random seed from the user's stored seed.
2. If the seed setting fails, logs a warning.
3. Applies `ORDER BY random()` with optional offset/limit.

## `apply_condition!`

`macros.rs:212` — Applies a parsed `Condition` to a query with optional negation.

```
apply_condition!($query, $expression, $filter, $condition)
```

**Negation truth table:**

| Condition | Not Negated | Negated |
|-----------|-------------|---------|
| `Values(v)` | `eq_any(v)` | `ne_all(v)` |
| `GreaterEq(v)` | `ge(v)` | `lt(v)` |
| `LessEq(v)` | `le(v)` | `gt(v)` |
| `Range(r)` | `between(r.start, r.end)` | `not_between(r.start, r.end)` |

## TEMP Table Cache Design

The filter cache uses two TEMP tables:
- **`matching`**: Stores IDs that match positive (non-negated) filters. Intersected per filter.
- **`nonmatching`**: Stores IDs that match unnegated versions of negative filters. Unioned per filter.

### Why This Design

Some filters (tag, pool, favorite, comment) require JOINs to other tables. You cannot stack these JOINs easily because each filter is independent. The TEMP tables let each filter run its own subquery. Then the final query filters by TEMP table membership.

### Cache Lifecycle

1. `CacheState` starts with `has_matching = false`, `has_nonmatching = false`, `completed = false`.
2. As each filter is processed, `update_filter_cache!` writes to the TEMP tables.
3. After all filters are processed, `apply_cache_filters!` applies the TEMP tables to the main query and sets `completed = true`.
4. Both TEMP tables use `ON COMMIT DROP` — they are automatically dropped at the end of the transaction.
