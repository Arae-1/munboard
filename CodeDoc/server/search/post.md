# post.rs — Post Search

This module provides search for posts. It has the most tokens and the most complex filter logic.

## Token Enum

`post.rs:28` — All search tokens for posts.

### Numeric Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `Id` | `id` | `post.id` | `i64` |
| `FileSize` | `file-size` | `post.file_size` | `i64` |
| `Width` | `width`, `image-width` | `post.width` | `i32` |
| `Height` | `height`, `image-height` | `post.height` | `i32` |
| `Area` | `area`, `image-area` | `post.width * post.height` | `i32` |
| `AspectRatio` | `ar`, `aspect-ratio`, `image-ar`, `image-aspect-ratio` | SQL expression (see below) | `f32` |
| `TagCount` | `tag-count` | `post_statistics.tag_count` | `i64` |
| `PoolCount` | `pool-count` | `post_statistics.pool_count` | `i64` |
| `CommentCount` | `comment-count` | `post_statistics.comment_count` | `i64` |
| `RelationCount` | `relation-count` | `post_statistics.relation_count` | `i64` |
| `NoteCount` | `note-count` | `post_statistics.note_count` | `i64` |
| `FavCount` | `fav-count` | `post_statistics.favorite_count` | `i64` |
| `FeatureCount` | `feature-count` | `post_statistics.feature_count` | `i64` |
| `Score` | `score` | `post_statistics.score` | `i64` |

### Enum Filters

| Token | Serialization | Column | Type |
|-------|--------------|--------|------|
| `Safety` | `rating`, `safety` | `post.safety` | `PostSafety` |
| `Type` | `type` | `post.type_` | `PostType` |

### String Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `Source` | `source` | `post.source` |
| `Description` | `description` | `post.description` |
| `Uploader` | `submit`, `upload`, `uploader` | `user.name` |

### Time Filters

| Token | Serialization | Column |
|-------|--------------|--------|
| `CreationTime` | `date`, `time`, `creation-date`, `creation-time` | `post.creation_time` |
| `LastEditTime` | `edit-date`, `edit-time`, `last-edit-date`, `last-edit-time` | `post.last_edit_time` |
| `LaunchDate` | `launch-date`, `launch-time` | `COALESCE(pool.launch_date, post.launch_date)` |
| `MediaCreatedDate` | `media-date`, `media-time` | `post.media_created_date` |
| `CommentTime` | `comment-date`, `comment-time` | `comment.creation_time` |
| `FavTime` | `fav-date`, `fav-time` | `post_favorite.time` |
| `FeatureTime` | `feature-date`, `feature-time` | `post_feature.time` |

### Special / Complex Filters

| Token | Serialization | Description |
|-------|--------------|-------------|
| `Tag` | `tag` | Posts with specific tags (the default anonymous token) |
| `TagCategory` | `tag-category` | Posts with tags in a category |
| `Pool` | `pool` | Posts in pools (by ID or name) |
| `PoolCategory` | `pool-category` | Posts in pools of a category |
| `Fav` | `fav` | Posts favorited by a user |
| `Comment` | `comment` | Posts with comments by a user |
| `NoteText` | `note-text` | Posts with notes containing text |
| `ContentChecksum` | `content-checksum` | Posts by exact checksum |
| `Flag` | `flag` | Posts by flag bits |
| `CustomField` | `cf`, `custom-field` | Posts by custom field value |
| `Special` | `special` | Special user-specific filters |

## `QueryBuilder` Struct

`post.rs:103` — The post search query builder.

### Constructor

#### `new(ctx, search_criteria) -> ApiResult<Self>`
`post.rs:242` — Creates a new builder. The anonymous token is `Token::Tag`. So bare words in a search are treated as tag filters.

#### `new_with_anonymous_token(ctx, search_criteria, anonymous_token) -> ApiResult<Self>`
`post.rs:246` — Same as `new` but with a custom anonymous token.

**Validation:** Rejects sorts on unsortable tokens: `ContentChecksum`, `NoteText`, `PoolCategory`, `Special`, `TagCategory`, `CustomField`.

### Count Optimization

`post.rs:116` — If no filters are applied and no user preferences exist, the count is read directly from `database_statistics.post_count` without running a query.

### Base Query

`post.rs:133` — The base query is:
```sql
SELECT post.id
FROM post
INNER JOIN post_statistics ON ...
LEFT JOIN user ON ...
```

## `AspectRatio` Calculation

`post.rs:291` — The aspect ratio is calculated as:
```sql
CAST(post.width AS REAL) / CAST(post.height AS REAL)
```
This uses `CAST` to avoid integer truncation.

## `ContentChecksum` Filter

`post.rs:299` — Filters posts by exact hash. Only `Values` condition is supported (no ranges, no wildcards). Each value must be a valid checksum string.

## `Flag` Filter

`post.rs:309` — Filters posts by flag bits. Multiple flags are OR'd together. The SQL uses bitwise AND:
```sql
post.flags & value != 0  -- for positive filter
post.flags & value == 0  -- for negated filter
```

## `Tag` Filter

`post.rs:323` — Filters posts by tag name.

**Positive filter (`tag:sky`):**
1. Joins `post_tag` to `tag_name` on `tag_id`.
2. Filters by tag name using `apply_str_filter!`.
3. Adds `DISTINCT` if multivalued.
4. Updates the matching TEMP table.

**Negative filter (`-tag:sky`):**
1. Collects all nonmatching post IDs into a union subquery.
2. Multiple negative tags are OR'd (union) into a single `NonmatchingPostTags` query.
3. At the end of `build_filtered`, the unioned nonmatching query is inserted into the `nonmatching` TEMP table in one batch.

This is an optimization. Instead of inserting nonmatching IDs for each negative tag individually, it unions them all into one insert.

## `TagCategory` Filter

`post.rs:358` — Filters posts by tag category name. Joins `tag`, `tag_category`, and `post_tag`. Results go through the TEMP table cache.

## `Pool` Filter

`post.rs:375` — Filters posts by pool. Tries to parse the condition as `i64` first (pool ID). If that fails, treats the condition as a pool name string.

## `PoolCategory` Filter

`post.rs:398` — Filters posts by pool category name. Joins `pool`, `pool_category`, and `pool_post`. Results go through the TEMP table cache.

## `Uploader` Filter

`post.rs:163` — Filters posts by uploader username. Uses `apply_str_filter!` directly on the `LEFT JOIN user` in the base query. Does NOT use TEMP tables.

## `Fav` (Favorite) Filter

`post.rs:415` — Filters posts by user who favorited them. Joins `post_favorite` to `user`. Results go through TEMP tables.

## `Comment` Filter

`post.rs:431` — Filters posts by user who commented on them. Joins `comment` to `user` with `DISTINCT`. Results go through TEMP tables.

## `NoteText` Filter

`post.rs:447` — Filters posts by note text content. Uses `DISTINCT`. Results go through TEMP tables.

## `CommentTime` Filter

`post.rs:459` — Filters posts that have comments within a time range. Joins `comment` with `DISTINCT`. Time filter on `comment.creation_time`. Results go through TEMP tables.

## `FavTime` Filter

`post.rs:471` — Filters posts that were favorited within a time range. Joins `post_favorite` with `DISTINCT`. Time filter on `post_favorite.time`. Results go through TEMP tables.

## `FeatureTime` Filter

`post.rs:486` — Filters posts that were featured within a time range. Joins `post_feature` with `DISTINCT`. Time filter on `post_feature.time`. Results go through TEMP tables.

## `Special` Filter

`post.rs:501` — Special user-specific filters. The condition must be a valid `SpecialToken`.

### `SpecialToken` Enum

`post.rs:278` — Values for the `special:` filter.

| Token | Serialization | Description |
|-------|--------------|-------------|
| `Liked` | `liked` | Posts the current user upvoted (`post_score.score = 1`) |
| `Disliked` | `disliked` | Posts the current user downvoted (`post_score.score = -1`) |
| `Fav` | `fav` | Posts the current user favorited |
| `Tumbleweed` | `tumbleweed` | Posts with zero favorites, zero comments, and zero scores |

Liked, Disliked, and Fav require a logged-in user (returns `NotLoggedIn` error for anonymous).

### Tumbleweed Logic

`post.rs:530` — Posts where:
- `favorite_count = 0`
- `comment_count = 0`
- `COUNT(post_score.post_id) = 0` (via HAVING clause; a score of 0 does not necessarily mean no ratings)

## `CustomField` Filter

`post.rs:545` — Filters posts by custom field values. The condition format is `name=value`.

**Logic:**
1. Splits condition on `=` to get `name` and `value_str`.
2. Parses `value_str` as `i64`.
3. Joins `post_custom_field` to `custom_field`.
4. Filters by `custom_field.name = name`.
5. Applies the value condition on `post_custom_field.value`.
6. Results go through TEMP tables.

## `LaunchDate` Filter

`post.rs:566` — Uses a PostgreSQL `COALESCE` expression:
```sql
COALESCE(
  (SELECT pool.launch_date FROM pool_post
   INNER JOIN pool ON pool.id = pool_post.pool_id
   WHERE pool_post.post_id = post.id AND pool.launch_date IS NOT NULL
   LIMIT 1),
  post.launch_date
)
```
This inherits the pool launch date if the post is in a pool with a launch date, otherwise uses the post's own launch date.

## Sorting

`post.rs:190` — Default sort is `Id` descending.

### Sortable Tokens

All tokens can be sorted except: `ContentChecksum`, `NoteText`, `PoolCategory`, `Special`, `TagCategory`, `CustomField`.

When sorting by `Tag`, it uses `post_statistics.tag_count`. Same for `Pool` (`pool_count`), `Fav` (`favorite_count`), `Comment` (`comment_count`).

### Random Sort

`post.rs:192` — If `random_sort` is true, the PostgreSQL seed is set to the user's search seed. All posts are ordered by `random()`. The user's seed is cycled to a new random value.

## Pagination

`post.rs:233` — If `extra_args` is set, OFFSET and LIMIT are applied to the final sorted query.

## User Preferences

`post.rs:184` — After all filters are applied, hidden posts (from user preferences) are excluded with a `NOT EXISTS` subquery.

## Default Token

The anonymous (default) token for post search is `Token::Tag`. A bare word in a post search is treated as a tag name filter.
