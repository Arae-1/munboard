# Database Admin Commands (`admin/database.rs`)

## Summary

This module provides three admin commands that operate on the database: renaming post files, caching thumbnail sizes, and recomputing database statistics.

## Imports

- `admin::{AdminResult, PRINT_INTERVAL, ProgressReporter}`: Admin result type and progress reporting.
- `app::AppState`: Application state.
- `content::hash::PostHash`: Post file path construction via hashing.
- `filesystem::Directory`: Directory path enum.
- `model::enums::MimeType`: MIME type detection.
- `schema::*`: All Diesel schema types for database tables (comment, pool, post, tag, user, and all statistics tables).
- `time::{DateTime, Timer}`: Time types and benchmarking.
- `admin`, `filesystem`: Module-level imports.
- `diesel::dsl::{count, max, sum}`: Diesel aggregate functions.
- `diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl}`: Diesel query builders.
- `std::ffi::OsStr`: OS string handling.
- `tracing::{error, warn}`: Logging.
- `walkdir::WalkDir`: Directory tree traversal.

## Function: `reset_filenames(state: &AppState)`

```rust
pub fn reset_filenames(state: &AppState)
```

Public entry point. Calls `reset_filenames_impl(state)`. If the implementation returns an error, logs it with `error!`. This function never panics.

## Function: `reset_filenames_impl(state: &AppState) -> AdminResult<()>`

Renames post files and thumbnails on disk. This is useful when the content hash changes (e.g., after recomputing checksums with a new hash algorithm).

**Algorithm:**

1. Start a `Timer` named "reset_filenames".
2. **Generated thumbnails directory** (`Directory::GeneratedThumbnails`):
   - Check if the directory exists. Skip if not.
   - Create a `ProgressReporter` named "Generated thumbnails renamed".
   - Walk the directory tree with `WalkDir::new()`.
   - For each file entry:
     - Check cancellation via `admin::is_cancelled()`.
     - Skip directories.
     - Extract `post_id` from the filename using `admin::get_post_id()`. Log an error and skip if extraction fails.
     - Compute `new_path` using `PostHash::new(&state.config, post_id).generated_thumbnail_path()`.
     - If the current path differs from the new path, call `filesystem::move_file(path, &new_path)`. Log an error on failure.
     - Call `progress.increment()`.
3. **Custom thumbnails directory** (`Directory::CustomThumbnails`):
   - Same algorithm as generated thumbnails but uses `PostHash::custom_thumbnail_path()` for the new path.
   - Progress reporter named "Custom thumbnails renamed".
4. **Posts directory** (`Directory::Posts`):
   - Same pattern as above.
   - Extract `post_id` from filename.
   - Determine the MIME type from the file path using `MimeType::from_path()`.
   - If MIME type is known: compute new path with `PostHash::content_path(mime_type)`.
   - If MIME type is unknown: warn about unsupported extension, default to `.png` extension, copy the original extension to the new path.
   - If paths differ, move the file, increment progress.
   - Progress reporter named "Posts renamed".
5. Return `Ok(())`.

Each directory section is independent. If one directory does not exist, the function continues to the next.

## Function: `reset_thumbnail_sizes(state: &AppState)`

```rust
pub fn reset_thumbnail_sizes(state: &AppState)
```

Public entry point. Calls `reset_thumbnail_sizes_impl(state)`. Logs errors on failure.

## Function: `reset_thumbnail_sizes_impl(state: &AppState) -> AdminResult<()>`

Updates database values for file sizes of avatars, generated thumbnails, and custom thumbnails.

**Algorithm:**

1. Start a `Timer` named "reset_thumbnail_sizes".
2. Get a blocking database connection from `state.connection_pool.get_blocking()`.
3. **Avatars directory** (`Directory::Avatars`):
   - Check if the directory exists.
   - Walk the directory tree.
   - For each file:
     - Check cancellation.
     - Skip directories.
     - Extract username from the file name.
     - Get `file_size` using `filesystem::file_size(path)`.
     - Update `user::custom_avatar_size` in the `user` table where `user::name` matches the username.
     - Execute the update and increment progress.
   - Progress reporter: "Avatar sizes cached".
4. **Generated thumbnails directory** (`Directory::GeneratedThumbnails`):
   - Same pattern.
   - Extract `post_id` from filename.
   - Get `file_size`.
   - Update `post::generated_thumbnail_size` in the `post` table where `post::id` matches.
   - Progress reporter: "Generated thumbnail sizes cached".
5. **Custom thumbnails directory** (`Directory::CustomThumbnails`):
   - Same pattern.
   - Update `post::custom_thumbnail_size` instead.
   - Progress reporter: "Custom thumbnails sizes cached".
6. Return `Ok(())`.

## Function: `reset_relation_stats(state: &AppState) -> AdminResult<()>`

Recomputes all database table statistics. This function is the core of the statistics recalculation. It processes every row one at a time, which makes it slow for large databases. A faster SQL-based version exists in `scripts/convert_szuru_database.sql`.

**Algorithm:**

1. Get a blocking database connection.
2. **Database-level counts:**
   - Count all rows in: `comment`, `pool`, `post`, `tag`, `user`.
   - Update `database_statistics` table with these five counts in a single UPDATE. This sets: `comment_count`, `pool_count`, `post_count`, `tag_count`, `user_count`.
3. **Comment statistics:**
   - Query: left-join `comment` with `comment_score`, group by `comment::id`, select `(comment::id, sum(comment_score::score))`.
   - For each `(comment_id, score)`:
     - Check cancellation.
     - Update `comment_statistics` where `id = comment_id`, set `score` to `score.unwrap_or(0)`.
     - Increment progress.
   - Progress reporter: "Comment statistics calculated".
4. **Pool category statistics:**
   - Query: left-join `pool_category` with `pool`, group by `pool_category::id`, select `(pool_category::id, count(pool::id))`.
   - For each `(category_id, usage_count)`:
     - Check cancellation.
     - Update `pool_category_statistics` where `id = category_id`, set `usage_count` to `usage_count.unwrap_or(0)`.
     - Increment progress.
   - Progress reporter: "Pool category statistics calculated".
5. **Pool statistics:**
   - Query: left-join `pool` with `pool_post`, group by `pool::id`, select `(pool::id, count(pool_post::post_id))`.
   - For each `(pool_id, post_count)`:
     - Check cancellation.
     - Update `pool_statistics` where `id = pool_id`, set `post_count` to `post_count.unwrap_or(0)`.
     - Increment progress.
   - Progress reporter: "Pool statistics calculated".
6. **Tag category statistics:**
   - Query: left-join `tag_category` with `tag`, group by `tag_category::id`, select `(tag_category::id, count(tag::id))`.
   - For each `(category_id, usage_count)`:
     - Check cancellation.
     - Update `tag_category_statistics` where `id = category_id`, set `usage_count` to `usage_count.unwrap_or(0)`.
     - Increment progress.
   - Progress reporter: "Tag category statistics calculated".
7. **Tag statistics (per tag):**
   - Collect all `tag::id` values.
   - For each `tag_id`:
     - Check cancellation.
     - `usage_count`: Count rows in `post_tag` where `tag_id = post_tag::tag_id`.
     - `implication_count`: Count rows in `tag_implication` where `child_id = tag_implication::child_id`.
     - `suggestion_count`: Count rows in `tag_suggestion` where `child_id = tag_suggestion::child_id`.
     - Update `tag_statistics` where `id = tag_id`, set all three counters.
     - Increment progress.
   - Progress reporter: "Tag statistics calculated".
8. **User statistics (per user):**
   - Collect all `user::id` values.
   - For each `user_id`:
     - Check cancellation.
     - `comment_count`: Count rows in `comment` where `user_id = comment::user_id`.
     - `favorite_count`: Count rows in `post_favorite` where `user_id = post_favorite::user_id`.
     - `upload_count`: Count rows in `post` where `user_id = post::user_id`.
     - Update `user_statistics` where `id = user_id`, set all three counters.
     - Increment progress.
   - Progress reporter: "User statistics calculated".
9. **Post statistics (per post):**
   - Collect all `post::id` values.
   - For each `post_id`:
     - Check cancellation.
     - `tag_count`: Count rows in `post_tag` where `post_id = post_tag::post_id`.
     - `pool_count`: Count rows in `pool_post` where `post_id = pool_post::post_id`.
     - `note_count`: Count rows in `post_note` where `post_id = post_note::post_id`.
     - `comment_count`: Count rows in `comment` where `post_id = comment::post_id`.
     - `relation_count`: Count rows in `post_relation` where `child_id = post_relation::child_id`.
     - `score`: Sum of `post_score::score` where `post_id = post_score::post_id`.
     - `favorite_count`: Count rows in `post_favorite` where `post_id = post_favorite::post_id`.
     - `feature_count`: Count rows in `post_feature` where `post_id = post_feature::post_id`.
     - `last_comment_time`: Max `comment::creation_time` where `post_id = comment::post_id`.
     - `last_favorite_time`: Max `post_favorite::time` where `post_id = post_favorite::post_id`.
     - `last_feature_time`: Max `post_feature::time` where `post_id = post_feature::post_id`.
     - Update `post_statistics` where `id = post_id`, set all eleven fields.
     - Increment progress.
   - Progress reporter: "Post statistics calculated".
10. Return `Ok(())`.

## Function: `reset_statistics(state: &AppState)`

```rust
pub fn reset_statistics(state: &AppState)
```

Public entry point. Calls `reset_statistics_impl(state)`. Logs errors on failure. This is the "reset statistics" admin task.

## Function: `reset_statistics_impl(state: &AppState) -> AdminResult<()>`

Recalculates cached file sizes, row counts, and all table statistics.

**Algorithm:**

1. Start a `Timer` named "reset_statistics".
2. Get a blocking database connection.
3. Reset `database_statistics::disk_usage` to 0. (Database triggers will increment it as file sizes are computed.)
4. **Posts directory** (`Directory::Posts`):
   - Check if the directory exists. Walk the tree.
   - For each file:
     - Check cancellation. Skip directories.
     - Extract `post_id` from filename. Log error and skip on failure.
     - Get `file_size` via `filesystem::file_size(path)`.
     - Update `post::file_size` in the `post` table where `post::id` matches.
     - Increment progress.
   - Progress reporter: "Posts content sizes cached".
5. Call `reset_thumbnail_sizes_impl(state)`. This caches avatar and thumbnail sizes.
6. Call `reset_relation_stats(state)`. This recomputes all table statistics.
7. Return the result of `reset_relation_stats`.

The order is important: `disk_usage` starts at 0, then post sizes update it via triggers, then thumbnail/avatar sizes update it further. Finally, all relation counts are recomputed.
