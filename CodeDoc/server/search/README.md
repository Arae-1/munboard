# Search Layer — README

## Architecture

The search layer converts user query strings into typed Diesel SQL queries. The framework core (`mod.rs`) defines the `Builder` trait, `SearchCriteria` struct, parsing engine, and flow control. Each entity module (post, tag, pool, comment, snapshot, user) implements a `QueryBuilder` that maps search tokens to SQL expressions.

## Module Structure

| Module | Doc | TEMP Tables | Base Tables |
|--------|-----|-------------|-------------|
| `mod` | [mod.md](mod.md) | — | — (framework core) |
| `macros` | [macros.md](macros.md) | — | — (query-building macros) |
| `parse` | [parse.md](parse.md) | — | — (string parsing engine) |
| `post` | [post.md](post.md) | Yes | post, post_statistics, user |
| `tag` | [tag.md](tag.md) | Yes | tag, tag_statistics, tag_category |
| `pool` | [pool.md](pool.md) | Yes | pool, pool_statistics, pool_category |
| `comment` | [comment.md](comment.md) | No | comment, comment_statistics, user |
| `snapshot` | [snapshot.md](snapshot.md) | No | snapshot, user |
| `user` | [user.md](user.md) | No | user |
| `preferences` | [preferences.md](preferences.md) | — | — (post-hiding filters) |
| `temp` | [temp.md](temp.md) | — | — (TEMP table declarations) |

## Query Flow (String to SQL)

1. The raw search string enters `SearchCriteria::new()`.
2. The string is split by unescaped whitespace into terms.
3. Each term is checked for `-` prefix (negation).
4. Each term is split on the first unescaped `:`:
   - **`sort:random`** → sets `random_sort = true`.
   - **`sort:<key>,<dir>`** → parses a sort directive with optional `asc`/`desc` direction.
   - **`<key>:<value>`** → parses a filter (key becomes a token, value stays raw).
   - **No `:`** → uses the entity's anonymous (default) token.
5. Filters remain as `UnparsedFilter` objects (token + raw condition string).
6. In `build_filtered()`, each filter is parsed lazily by the entity-specific query builder:
   - `apply_filter!` — integer/enum conditions.
   - `apply_str_filter!` — string conditions with wildcard support.
   - `apply_time_filter!` — time/date range conditions.
7. Filters that require JOINs use TEMP tables for isolation.
8. After all filters, `apply_cache_filters!` applies TEMP table results.
9. User preferences (post hiding) are applied as `WHERE NOT EXISTS`.
10. Sorting, offset, and limit are applied to produce the final ordered ID list.

## Search String Syntax

### Token Types

| Type | Syntax | Example |
|------|--------|---------|
| Anonymous (bare word) | `<value>` | `sky` (uses entity default token) |
| Named filter | `<key>:<value>` | `tag:sky`, `score:5..` |
| Sort style | `sort:<key>,<dir>` | `sort:id,asc`, `sort:score` |
| Sort random | `sort:random` | `sort:random` |
| Special | `special:<value>` | `special:liked` (posts only) |
| Custom field | `cf:<name>=<value>` | `cf:rating=5` (posts only) |

### Value Formats

| Format | Example | Meaning |
|--------|---------|---------|
| Single value | `5` | Exactly 5 |
| Comma list | `1,3,5` | Any of 1, 3, or 5 |
| Range open-end | `5..` | Greater than or equal to 5 |
| Range open-start | `..5` | Less than or equal to 5 |
| Range closed | `5..10` | Between 5 and 10 (inclusive) |
| Wildcard | `star*` | LIKE pattern (`*` = `%`) |

### Negation

Prepend `-` to negate any filter: `-tag:sky`, `-score:5..`, `-sort:id`. For sorts, negation inverts the direction (`-sort:id,asc` → `sort:id,desc`).

### Time Values

| Input | Parsed Range |
|-------|-------------|
| `today` | Midnight today to midnight tomorrow |
| `yesterday` | Midnight yesterday to midnight today |
| `2024` | All of 2024 |
| `2024-7` | All of July 2024 |
| `2024-7-11` | July 11, 2024 |

Time values are always parsed as ranges. When used as equality filters, a `tstzmultirange` contain-check is built. When used as range endpoints, the range's start/end boundaries are used.

### Wildcards

Fields that accept text (names, descriptions) accept `*` as a wildcard. Internally converted to SQL `%`. The system also escapes SQL's `_` wildcard and lowercases the pattern to force PostgreSQL to use TEXT indexes instead of CITEXT indexes.

### Escapes

Use `\` before special characters:

| Input | Meaning |
|-------|---------|
| `\ ` | Literal space (not a term separator) |
| `\,` | Literal comma (not a value separator) |
| `\:` | Literal colon (not a key:value separator) |
| `\.` | Literal dot (not a range separator) |
| `\\` | Literal backslash |

Example: `name\:my\ tag\ with\,comma` → key `name` with value `my tag with,comma`.

## Entity-Specific Default Tokens

| Entity | Bare Word Meaning |
|--------|-------------------|
| Post | tag name filter |
| Tag | tag name filter |
| Pool | pool name filter |
| Comment | comment text filter |
| Snapshot | resource type filter |
| User | username filter |

## Sorting System

- Default sort is entity-specific. Posts default to `id` descending. Tags/pools/comments/snapshots default to `creation_time` descending. Users default to `name` descending.
- Sort direction uses `,asc` or `,desc`. Default direction is descending.
- Multiple sorts chain via `then_order_by`. The first unlabeled sort replaces the default sort; `sort:random` replaces all.
- `Desc` uses `nulls_last()` for NULL-safe ordering.
- Random sort sets PostgreSQL's random seed from the user's stored `search_seed`, then cycles it. Anonymous users use seed `0.0`.
- Some tokens are unsortable: `ContentChecksum`, `NoteText`, `PoolCategory`, `Special`, `TagCategory`, `CustomField` (post search rejects sorts on these).

## Pagination

All entity builders support `set_offset_and_limit(offset, limit)`. These are applied as SQL `OFFSET` and `LIMIT` on the final sorted query. List endpoint handlers cap `limit` at 1000.

## Count Optimization

When no filters are applied and no user preferences exist, the builder reads the total count directly from `database_statistics`:

| Entity | Statistics Column |
|--------|-------------------|
| Post | `post_count` |
| Tag | `tag_count` |
| Pool | `pool_count` |
| Comment | `comment_count` |
| User | `user_count` |
| Snapshot | Always runs count query |

## TEMP Table Filter Cache

Complex filters that require JOINs to other tables use session-local TEMP tables:

- **`matching`** — Stores IDs for positive (non-negated) filters. Intersected (AND'd) across filters.
- **`nonmatching`** — Stores IDs for unnegated versions of negative filters. Unioned (OR'd) across filters.

Post search has a special optimization: all negative tag filters are unioned into a single subquery before insertion into `nonmatching`, instead of inserting one at a time.

TEMP tables use `ON COMMIT DROP` — automatically dropped at transaction end.

## User Preferences and Post Hiding

The `preferences` module builds a `WHERE NOT EXISTS` subquery to hide posts:

- **Restricted users** cannot see unsafe posts.
- **Anonymous users** with configured preferences cannot see: sketchy posts (if `hide_sketchy`), unsafe posts (if `hide_unsafe`), untagged posts (if `hide_untagged`), posts with blacklisted tags, or posts with tags in blacklisted categories.
- Regular and higher-ranked users see all posts.
- Comments on hidden posts are also hidden from comment search results.

## Quick Reference — All Search Tokens

### Post Search (37 tokens)

| Category | Tokens |
|----------|--------|
| Numeric | `id`, `file-size`, `width`/`image-width`, `height`/`image-height`, `area`/`image-area`, `ar`/`aspect-ratio`/`image-ar`/`image-aspect-ratio`, `tag-count`, `pool-count`, `comment-count`, `relation-count`, `note-count`, `fav-count`, `feature-count`, `score` |
| Enum | `safety`/`rating`, `type` |
| String | `source`, `description`, `uploader`/`upload`/`submit` |
| Time | `date`/`time`/`creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time`, `launch-date`/`launch-time`, `media-date`/`media-time`, `comment-date`/`comment-time`, `fav-date`/`fav-time`, `feature-date`/`feature-time` |
| Complex | `tag`, `tag-category`, `pool`, `pool-category`, `fav`, `comment`, `note-text`, `content-checksum`, `flag`, `cf`/`custom-field`, `special` |
| Special | `special:liked`, `special:disliked`, `special:fav`, `special:tumbleweed` |

### Tag Search (10 tokens)

| Category | Tokens |
|----------|--------|
| String | `name`, `category`, `description` |
| Numeric | `usages`/`usage-count`/`post-count`, `implication-count`, `suggestion-count` |
| Time | `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time` |
| Relation | `implies`, `suggests` |

### Pool Search (5 tokens)

| Category | Tokens |
|----------|--------|
| String | `name`, `category` |
| Numeric | `post-count` |
| Time | `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time` |

### Comment Search (7 tokens)

| Category | Tokens |
|----------|--------|
| Numeric | `id`, `post`, `score` |
| String | `text`, `user`/`author` |
| Time | `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time` |

### Snapshot Search (5 tokens)

| Category | Tokens |
|----------|--------|
| String | `id` (resource ID), `user` |
| Enum | `operation`, `resource-type` |
| Time | `date`/`time` |

### User Search (3 tokens)

| Category | Tokens |
|----------|--------|
| String | `name` |
| Time | `creation-date`/`creation-time`, `last-login-date`/`last-login-time`/`login-date`/`login-time` |

## Documentation Files

| File | Content |
|------|---------|
| [mod.md](mod.md) | Framework core: `Builder` trait, `SearchCriteria`, `Condition`, `StrCondition`, `Order`, sorting, random seed, query parsing flow, negation logic |
| [macros.md](macros.md) | Query-building macros: `apply_filter!`, `apply_str_filter!`, `apply_time_filter!`, `apply_sort!`, `apply_condition!`, TEMP table cache macros |
| [parse.md](parse.md) | String parsing: `str_condition`, `time_condition`, `condition`, escape handling, wildcard conversion, time parsing, range splitting |
| [post.md](post.md) | Post query builder: 37 tokens, TEMP table usage, aspect ratio, checksum, flag, tag/pool/fav/comment/none-text filters, special tokens, custom fields, launch date coalesce |
| [tag.md](tag.md) | Tag query builder: 10 tokens, anonymous blacklist injection, implication/suggestion filters |
| [pool.md](pool.md) | Pool query builder: 5 tokens, name filter with TEMP tables |
| [comment.md](comment.md) | Comment query builder: 7 tokens, no TEMP tables, user preferences integration |
| [snapshot.md](snapshot.md) | Snapshot query builder: 5 tokens, `ResourceOperation` and `ResourceType` enum filters |
| [user.md](user.md) | User query builder: 3 tokens, simplest base query (single table) |
| [preferences.md](preferences.md) | User preference filters: `hidden_posts`, `has_preferences`, safety hiding, tag/category blacklist, aliased table subquery |
| [temp.md](temp.md) | TEMP table declarations: `matching` and `nonmatching` tables, `ON COMMIT DROP`, compatibility declarations |
