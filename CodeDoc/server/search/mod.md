# mod.rs — Search Framework Core

This module defines the core search framework. All entity search modules (post, tag, pool, comment, snapshot, user) use this framework.

## Module Structure

| Submodule  | Visibility | Purpose |
|------------|------------|---------|
| `comment`  | pub        | Comment search |
| `macros`   | private    | Diesel query-building macros |
| `parse`    | private    | Query string parsing |
| `pool`     | pub        | Pool search |
| `post`     | pub        | Post search |
| `preferences` | pub    | User preference filters |
| `snapshot` | pub        | Snapshot search |
| `tag`      | pub        | Tag search |
| `temp`     | private    | TEMP tables for filter caching |
| `user`     | pub        | User search |

## `Builder` Trait

`mod.rs:22` — Every search module uses this trait. It defines the interface for a search query builder.

### Associated Types

- **`Token`**: An enum. Each variant is a search keyword.
- **`BoxedQuery`**: The Diesel boxed query type for the entity.

### Methods

#### `criteria(&mut self) -> &mut SearchCriteria<'a, Self::Token>`
Returns a mutable reference to the stored search criteria.

#### `set_offset_and_limit(&mut self, offset: i64, limit: i64)`
`mod.rs:30` — Sets the `OFFSET` and `LIMIT` for the SQL query. Stores them in `extra_args`.

#### `list(&mut self, conn: &mut PgConnection) -> ApiResult<(i64, Vec<i64>)>`
`mod.rs:36` — Runs both a count query and a load query. Returns `(total_count, Vec<id>)`. If `random_sort` is true, it changes the user's search seed before running queries.

#### `load(&mut self, conn: &mut PgConnection) -> ApiResult<Vec<i64>>`
`mod.rs:47` — Builds a filtered query, then gets ordered primary key IDs. Returns a vector of IDs.

#### `count(&mut self, conn: &mut PgConnection) -> ApiResult<i64>`
Counts the number of rows that match the search criteria.

#### `build_filtered(&mut self, conn: &mut PgConnection) -> ApiResult<Self::BoxedQuery>`
Builds the WHERE clause from all parsed filters. Returns a boxed Diesel query.

#### `get_ordered_ids(&self, conn: &mut PgConnection, query: Self::BoxedQuery) -> QueryResult<Vec<i64>>`
Applies sorting, offset, and limit to the filtered query. Returns ordered IDs.

## `TimeParsingError` Enum

`mod.rs:64` — Error type for time parsing failures.

| Variant | Description |
|---------|-------------|
| `TooFewArgs` | Date string is empty |
| `TooManyArgs` | Date string has more than 3 parts (year-month-day) |
| `NotAnInteger` | A date component is not a valid integer |
| `OutOfRange` | A date component is out of valid range |

## Maximum Query Length

`mod.rs:80` — `MAX_QUERY_LEN` is the maximum length of a search query string. Its value is 4096 bytes.

- The limit is measured in bytes (`str::len()`), not characters. The limit exists to cap parsing cost and memory, which scale with the encoded size, and byte length is a constant-time check.
- 4096 bytes allows queries with hundreds of space-separated terms while bounding the cost of token splitting and the size of the generated temporary filter tables.
- `SearchCriteria::new` checks the limit before any parsing work. A longer query returns `ApiError::QueryTooLong`, which has HTTP status 400 Bad Request.

## `SearchCriteria<'a, T>` Struct

`mod.rs:84` — The core struct that holds all parsed search parameters.

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `ctx` | `&'a Context` | Reference to the application context |
| `filters` | `Vec<UnparsedFilter<'a, T>>` | List of unparsed filters |
| `sorts` | `Vec<ParsedSort<T>>` | List of parsed sort directives |
| `random_sort` | `bool` | True if `sort:random` was specified |
| `extra_args` | `Option<QueryArgs>` | Offset and limit for pagination |

### Methods

#### `has_sort(&self) -> bool`
`mod.rs:93` — Returns true if any sorts are set or `random_sort` is true.

#### `has_filter(&self) -> bool`
`mod.rs:97` — Returns true if any filters are set.

#### `new(ctx, search_criteria, anonymous_token) -> ApiResult<Self>`
`mod.rs:110` — Parses a search query string into filters and sorts.

**Parsing logic:**

1. If the query string is longer than `MAX_QUERY_LEN` (4096 bytes), the function returns `ApiError::QueryTooLong` (HTTP 400 Bad Request). This check runs before any parsing work, so pathological input fails fast with a clean 400 instead of driving the splitting logic.
2. The search string is split by unescaped whitespace into terms.
3. Each term is checked for a `-` prefix. If present, the filter is negated.
4. Each term is then split on the first unescaped `:` character:
   - **`sort:random`** → Sets `random_sort = true`.
   - **`sort:<key>,<dir>`** → Parses a sort. The key is after `sort:`. The direction is after `,`. Valid directions: `asc`, `desc`. If no direction, defaults to `Desc`. If the sort is negated (`-sort:`), the direction is inverted.
   - **`<key>:<condition>`** → Parses a filter. The key before `:` is parsed as a token. The condition after `:` is stored as the filter condition string.
   - **No `:`** → Uses the `anonymous_token` as the key and the entire term as the condition.

## `QueryArgs` Struct

`mod.rs:170` — Stores pagination parameters.

| Field | Type | Description |
|-------|------|-------------|
| `offset` | `i64` | Number of rows to skip |
| `limit` | `i64` | Maximum number of rows to return |

## `Order` Enum

`mod.rs:176` — Sort direction.

| Variant | Default | Description |
|---------|---------|-------------|
| `Asc` | ❌ | Ascending order |
| `Desc` | ✅ | Descending order (default) |

The `Not` trait is used to invert direction: `!Asc = Desc`, `!Desc = Asc`.

## `Condition<V>` Enum

`mod.rs:194` — A parsed filter condition for a typed column.

| Variant | Description | Query Syntax |
|---------|-------------|-------------|
| `Values(Vec<V>)` | Match any of the listed values | `a,b,c` |
| `GreaterEq(V)` | Greater than or equal to value | `a..` |
| `LessEq(V)` | Less than or equal to value | `..b` |
| `Range(Range<V>)` | Between two values (inclusive) | `a..b` |

## `StrCondition` Enum

`mod.rs:205` — A parsed filter condition for a text column.

| Variant | Description |
|---------|-------------|
| `Regular(Condition<String>)` | Standard condition (values, range, greater, less) |
| `WildCard(String)` | LIKE pattern (uses `*` as wildcard, converted to `%`) |

## `UnparsedFilter<'a, T>` Struct

`mod.rs:212` — An unparsed filter from the query string.

| Field | Type | Description |
|-------|------|-------------|
| `kind` | `T` | Parsed token identifying the column/expression |
| `condition` | `&'a str` | Raw condition string (not yet parsed) |
| `negated` | `bool` | True if the term started with `-` |

### Methods

#### `unnegated(self) -> Self`
`mod.rs:220` — Returns a copy with `negated` set to false.

## `ParsedSort<T>` Struct

`mod.rs:231` — A parsed sort directive.

| Field | Type | Description |
|-------|------|-------------|
| `kind` | `T` | Token identifying the column to sort by |
| `order` | `Order` | Sort direction |

## `CacheState` Struct

`mod.rs:240` — Tracks the state of the TEMP table filter cache.

| Field | Type | Description |
|-------|------|-------------|
| `has_matching` | `bool` | True if the `matching` TEMP table has rows |
| `has_nonmatching` | `bool` | True if the `nonmatching` TEMP table has rows |
| `completed` | `bool` | True if all filters have been processed |

## SQL Functions

### `random() -> BigInt`
`mod.rs:258` — PostgreSQL `random()` function declaration for Diesel.

### `lower<T: SingleValue>(text: T) -> Text`
`mod.rs:263` — PostgreSQL `lower()` function declaration. Used with wildcard filters to make PostgreSQL use TEXT indexes instead of CITEXT indexes.

## Random Sort Seed Functions

### `set_seed(conn, client) -> QueryResult<()>`
`mod.rs:267` — Sets the PostgreSQL global random seed. For anonymous users, the seed is `0.0`. For logged-in users, the seed is the user's `search_seed` value from the `user` table.

### `change_user_seed(conn, client) -> QueryResult<()>`
`mod.rs:281` — Generates a new random search seed for the user. Uses `OsRng` to generate a random `i16`, then converts it to an `f32` between `-1.0` and `1.0`. Only changes seeds for logged-in users.

## Query Parsing Flow

1. `SearchCriteria::new` rejects the raw search string if it is longer than `MAX_QUERY_LEN` (4096 bytes). The error is `ApiError::QueryTooLong` (HTTP 400 Bad Request).
2. The raw search string is split by unescaped whitespace into terms.
3. Each term is checked for negation (`-` prefix).
4. Each term is split on the first unescaped `:`.
5. If the left side is `sort`, the term becomes a sort directive.
6. If the left side is anything else, the term becomes a filter. The left side is parsed as a token. The right side is stored as the raw condition string.
7. If there is no `:`, the entire term is an anonymous filter (uses the default token).
8. Filters remain unparsed. They are parsed later during `build_filtered` when `apply_filter!`, `apply_str_filter!`, or `apply_time_filter!` is called.

## Negation Logic

A `-` prefix flips the meaning of a filter:

| Original Condition | Negated SQL |
|-------------------|-------------|
| `eq_any(a,b,c)` | `ne_all(a,b,c)` |
| `ge(x)` | `lt(x)` |
| `le(x)` | `gt(x)` |
| `between(a, b)` | `not_between(a, b)` |

For sorts, `-sort:id,asc` becomes `sort:id,desc` (the `!` operator inverts the direction).
