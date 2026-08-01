# Post Admin Operations (`admin/post.rs`)

## Summary

This module provides admin commands that operate on posts: integrity checking, checksum recomputation, signature recomputation, index rebuilding, post type recomputation, and thumbnail regeneration. All operations run in parallel using Rayon.

## Imports

- `admin::input::PostEditor`: Post query editor.
- `admin::{AdminResult, PRINT_INTERVAL, ProgressReporter, input}`: Admin utilities.
- `app::AppState`: Application state.
- `content::hash::{Checksum, PostHash}`: Content checksums and file paths.
- `content::signature::SIGNATURE_VERSION`: Current signature algorithm version.
- `content::thumbnail::{ThumbnailCategory, ThumbnailType}`: Thumbnail types.
- `content::{decode, hash, signature, thumbnail}`: Content processing functions.
- `model::enums::MimeType`: MIME type enum.
- `model::post::{CompressedSignature, NewPostSignature}`: Post signature model types.
- `schema::{database_statistics, post, post_signature}`: Diesel schema types.
- `search::Builder`: Search query builder trait.
- `search::post::{QueryBuilder, Token}`: Post search query builder.
- `time::{DateTime, Timer}`: Time types.
- `admin`, `update`: Module-level imports.
- `diesel::dsl::exists`: SQL EXISTS query.
- `diesel::{Connection, ExpressionMethods, Insertable, OptionalExtension, QueryDsl, RunQueryDsl}`: Diesel ORM.
- `rayon::iter::{IntoParallelIterator, ParallelIterator}`: Parallel iteration.
- `tracing::{error, info, warn}`: Logging.

## Function: `check_integrity(state, editor)`

```rust
pub fn check_integrity(state: &AppState, editor: &mut PostEditor)
```

Checks the integrity of post files by comparing stored checksums with current file checksums. This detects file corruption or silent modification.

**Algorithm:**
1. Enter `input::user_input_loop` with the `PostEditor`.
2. In the closure:
   - Call `user_query(state, editor)` to get a `Vec<i64>` of post IDs.
   - Start a `Timer` named "check_integrity".
   - Create two progress reporters:
     - `progress`: "Posts checked" (prints every `PRINT_INTERVAL`).
     - `failures`: "Integrity checks failed" (never auto-prints; reported at end).
   - Convert `post_ids` to a parallel iterator with `.into_par_iter()`.
   - Call `.try_for_each(|post_id| check_integrity_in_parallel(state, post_id, &progress, &failures))`.
   - Report the final failure count with `failures.report()`.
   - Return `Ok(())`.

## Function: `recompute_checksums(state, editor)`

```rust
pub fn recompute_checksums(state: &AppState, editor: &mut PostEditor)
```

Recomputes post checksums. Useful when the checksum algorithm changes.

**Algorithm:**
1. Enter `input::user_input_loop`.
2. Get post IDs from `user_query`.
3. Start a `Timer` named "recompute_checksums".
4. Create progress reporters: "Checksums computed" and "Duplicates found".
5. Parallel-iterate over post IDs, calling `recompute_checksum_in_parallel`.
6. Report duplicate count at the end.

## Function: `recompute_signatures(state, editor)`

```rust
pub fn recompute_signatures(state: &AppState, editor: &mut PostEditor)
```

Recomputes both post signatures and signature indexes. Useful when signature parameters change.

**Algorithm:**
1. Enter `input::user_input_loop`.
2. Get post IDs from `user_query`.
3. **Important:** Before recomputing, update `database_statistics::signature_version` to the current `SIGNATURE_VERSION`. This is done first so the server can continue running during computation.
4. Start a `Timer` named "recompute_signatures".
5. Create progress reporter: "Signatures computed".
6. Parallel-iterate over post IDs, calling `recompute_signature_in_parallel`.

## Function: `recompute_indexes(state, editor)`

```rust
pub fn recompute_indexes(state: &AppState, editor: &mut PostEditor)
```

Recomputes post signature indexes only (not the signatures themselves). Much faster than `recompute_signatures` because it reads signatures from the database rather than reading post content from disk.

**Algorithm:**
1. Enter `input::user_input_loop`.
2. Get post IDs from `user_query`.
3. Start a `Timer` named "recompute_indexes".
4. Create progress reporter: "Indexes computed".
5. Parallel-iterate over post IDs, calling `recompute_index_in_parallel`.

## Function: `recompute_post_types(state, editor)`

```rust
pub fn recompute_post_types(state: &AppState, editor: &mut PostEditor)
```

Recomputes post types. Applies changes to existing posts when post type definitions change (e.g., a new format is added).

**Algorithm:**
1. Enter `input::user_input_loop`.
2. Get post IDs.
3. Start `Timer` named "recompute_post_types".
4. Progress reporter: "Post types computed".
5. Parallel-iterate calling `recompute_post_type_in_parallel`.

## Function: `regenerate_thumbnails(state, editor)`

```rust
pub fn regenerate_thumbnails(state: &AppState, editor: &mut PostEditor)
```

Regenerates post thumbnails.

**Algorithm:**
1. Enter `input::user_input_loop`.
2. Get post IDs.
3. Start `Timer` named "regenerate_thumbnails".
4. Progress reporter: "Thumbnails regenerated".
5. Parallel-iterate calling `regenerate_thumbnail_in_parallel`.

## Function: `check_integrity_in_parallel(state, post_id, progress, failures)`

```rust
fn check_integrity_in_parallel(
    state: &AppState,
    post_id: i64,
    progress: &ProgressReporter,
    failures: &ProgressReporter,
) -> AdminResult<()>
```

Checks content integrity for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation via `admin::is_cancelled()`.
2. Get a blocking database connection.
3. Query the `post` table for `(mime_type, checksum)` by `post_id`.
   - `Ok(Some(metadata))`: Continue with the metadata.
   - `Ok(None)`: Post was deleted after the task started. Return `Ok(())` (skip).
   - `Err(err)`: Log error and return `Ok(())` (skip, do not abort the batch).
4. Construct the content path: `PostHash::new(&state.config, post_id).content_path(mime_type)`.
5. Compute the current file checksum: `hash::compute_checksums(&content_path)`.
   - On success: destructure `(checksum, _)` (ignore MD5).
   - On failure: Log error and return `Ok(())`.
6. Compare `checksum` (from database) with `file_checksum` (from disk).
   - If they differ: Log a warning. Call `failures.increment()`.
7. Call `progress.increment()`.
8. Return `Ok(())`.

## Function: `recompute_index_in_parallel(state, post_id, progress)`

```rust
fn recompute_index_in_parallel(state: &AppState, post_id: i64, progress: &ProgressReporter) -> AdminResult<()>
```

Recomputes the search index for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation.
2. Get a blocking database connection.
3. Read `CompressedSignature` from `post_signature` table by `post_id`. Uses `.for_no_key_update()` for a row-level lock. Unwraps the result (must exist).
4. Call `signature::generate_indexes(&signature)` to produce the word indexes.
5. Update `post_signature` table: set `words` to the new indexes.
6. On success: `progress.increment()`. On failure: log error.
7. Return `Ok(())`.

## Function: `recompute_checksum_in_parallel(state, post_id, progress, duplicate_count)`

```rust
fn recompute_checksum_in_parallel(
    state: &AppState,
    post_id: i64,
    progress: &ProgressReporter,
    duplicate_count: &ProgressReporter,
) -> AdminResult<()>
```

Recomputes the checksum for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation.
2. Get a blocking database connection.
3. Query `post::mime_type` for `post_id`.
   - `Ok(Some(mime_type))`: Continue.
   - `Ok(None)`: Post deleted, skip.
   - `Err(err)`: Log and skip.
4. Construct content path via `PostHash::content_path(mime_type)`.
5. Compute checksums: `hash::compute_checksums(&image_path)`.
   - Destructure into `(checksum, md5_checksum)`.
   - On failure: Log and skip.
6. **Duplicate check:** Query the `post` table for another post with the same `checksum` but different `post_id`.
   - `Err`: Log and skip.
   - `Some(dup_id)`: Warn about potential duplicate. Call `duplicate_count.increment()`. Return `Ok(())` (skip the update).
7. **Update:** Update the `post` table for `post_id`:
   - Set `checksum` (BLAKE3).
   - Set `checksum_md5` (MD5).
   - Set `last_edit_time` to `DateTime::now()`.
8. On success: `progress.increment()`. On failure: log error.
9. Return `Ok(())`.

## Function: `recompute_signature_in_parallel(state, post_id, progress)`

```rust
fn recompute_signature_in_parallel(state: &AppState, post_id: i64, progress: &ProgressReporter) -> AdminResult<()>
```

Recomputes the signature and search indexes for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation.
2. Get a blocking database connection.
3. Query `post::mime_type` for `post_id`. Skip if not found or error.
4. Construct content path via `PostHash::content_path(mime_type)`.
5. Decode a representative image: `decode::representative_image(&state.config, &content_path, mime_type)`.
   - On failure: Log error and return `Ok(())` (skip).
6. Compute the signature: `signature::compute(&image)`.
7. Generate search indexes: `signature::generate_indexes(&image_signature)`.
8. **Database transaction:**
   - Check `post` exists via `diesel::select(exists(post::table.find(post_id)))`. If not, return.
   - Check `post_signature` exists via `exists`.
   - If signature exists: UPDATE `post_signature` table with new signature and words.
   - If signature does not exist: INSERT new `NewPostSignature` with `post_id`, `signature`, `words`.
9. On success: `progress.increment()`. On failure: log error.
10. Return `Ok(())`.

## Function: `recompute_post_type_in_parallel(state, post_id, progress)`

```rust
fn recompute_post_type_in_parallel(state: &AppState, post_id: i64, progress: &ProgressReporter) -> AdminResult<()>
```

Recomputes the post type for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation.
2. Get a blocking database connection.
3. Query `post::mime_type` for `post_id`. Skip if not found or error.
4. Construct content path via `PostHash::content_path(mime_type)`.
5. Detect post type: `decode::detect_post_type(&content_path, mime_type)`. Skip on error.
6. Update `post::type_` in the `post` table to the new type.
7. On success: `progress.increment()`. On failure: log error.
8. Return `Ok(())`.

## Function: `regenerate_thumbnail_in_parallel(state, post_id, progress)`

```rust
fn regenerate_thumbnail_in_parallel(state: &AppState, post_id: i64, progress: &ProgressReporter) -> AdminResult<()>
```

Regenerates the thumbnail for a single post. Designed for parallel execution.

**Algorithm:**
1. Check cancellation.
2. Get a blocking database connection.
3. Query `post::mime_type` for `post_id`. Skip if not found or error.
4. Construct content path via `PostHash::content_path(mime_type)`.
5. Decode representative image: `decode::representative_image(...)`.
   - On failure: Log error and skip.
6. Create thumbnail: `thumbnail::create(&state.config, &image, ThumbnailType::Post)`.
7. Save thumbnail: `update::post::thumbnail(&mut conn, &post_hash, &thumbnail, ThumbnailCategory::Generated)`.
   - On failure: Log error.
   - On success: `progress.increment()`.
8. Return `Ok(())`.

## Function: `user_query(state, editor) -> AdminResult<Vec<i64>>`

```rust
fn user_query(state: &AppState, editor: &mut PostEditor) -> AdminResult<Vec<i64>>
```

Prompts the user for a post selection query and returns matching post IDs.

**Algorithm:**
1. Enter a loop for retry on parse failures.
2. Create a `Ctx` from `state.clone().make_context(admin::client())`.
3. Call `input::read("Select posts...", editor)` to get user input.
4. Parse the input with `QueryBuilder::new_with_anonymous_token(&ctx, &user_input, Token::Id)`.
   - On error: Log parse error and loop again.
   - On success `builder`:
     - Get a blocking database connection.
     - Execute in a transaction: `conn.transaction(|conn| builder.load(conn))` to get post IDs.
     - Log the number of posts found: `info!("Found {} posts", post_ids.len())`.
     - Return the IDs.
