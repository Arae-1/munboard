# Post API — `post.rs`

File: `server/src/api/post.rs`

## Overview

This module defines all post-related HTTP endpoints. It handles CRUD operations, search, favorite, rating, featured posts, reverse image search, and post merging. File uploads use multipart/form-data with a default body limit of `MAX_UPLOAD_SIZE`.

---

## Router Registration

### `routes() -> OpenApiRouter<AppState>`

**Location:** `post.rs:42`

Returns a router with these endpoints:

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/posts` | `list` |
| `GET` | `/post/{id}` | `get` |
| `GET` | `/post/{id}/around` | `get_neighbors` |
| `GET` | `/featured-post` | `get_featured` |
| `POST` | `/featured-post` | `feature` |
| `POST` | `/posts/reverse-search` | `reverse_search` |
| `POST` | `/posts` | `create` |
| `POST` | `/post-merge` | `merge` |
| `POST` | `/post/{id}/favorite` | `favorite` |
| `DELETE` | `/post/{id}/favorite` | `unfavorite` |
| `PUT` | `/post/{id}/score` | `rate` |
| `PUT` | `/post/{id}` | `update` |
| `DELETE` | `/post/{id}` | `delete` |

The `reverse_search`, `create`, and `update` routes use `DefaultBodyLimit::max(MAX_UPLOAD_SIZE)` for file uploads.

---

## Constants and Globals

| Name | Value/Type | Purpose |
|------|-----------|---------|
| `MAX_POSTS_PER_PAGE` | `1000` | Maximum posts per page in list queries |
| `POST_TAG_MUTEX` | `LazyLock<AsyncMutex<()>>` | Prevents concurrent tag updates on posts |

---

## Helper: `tagging_update`

**Location:** `post.rs:85`

```rust
async fn tagging_update<T, F>(connection_pool, tags_updated, update) -> ApiResult<T>
```

If `tags_updated` is `true`, it acquires the `POST_TAG_MUTEX` lock. Then it runs the `update` closure inside a database transaction on a fresh connection. This prevents:
- Uniqueness violations when two updates share a new tag
- Deadlocks from statistics updates when two updates share an existing tag

---

## Helper: `verify_visibility`

**Location:** `post.rs:1330`

```rust
fn verify_visibility(conn, ctx, post_id) -> ApiResult<()>
```

1. Checks if the post exists in the database (`post::table.find(post_id)`).
2. If not found, returns `ApiError::NotFound(ResourceType::Post)`.
3. Applies anonymous user preferences: checks if the post is hidden due to tag or tag-category blacklists.
4. If hidden, returns `ApiError::Hidden(ResourceType::Post)`.

---

## Endpoints

### 1. `GET /posts` — List Posts

**Handler:** `list` at `post.rs:189`

**Authentication:** Requires `Action::PostList`.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query with named tokens, sort tokens, and special tokens |
| `fields` | `Mask<Field>` | Comma-separated field selection |
| `offset` | `Option<i64>` | Starting position (default: 0) |
| `limit` | `NonZeroI64` | Maximum results (capped at 1000) |

**Search Tokens (Named):**

| Token | Description |
|-------|-------------|
| `id` | Post number |
| `tag` | Tag name (wildcards allowed) |
| `tag-category` | Tag category (wildcards allowed) |
| `score` | Post score |
| `uploader`, `upload`, `submit` | Uploader name (wildcards allowed) |
| `comment` | Commented by user (wildcards allowed) |
| `fav` | Favorited by user (wildcards allowed) |
| `pool` | Pool name (wildcards allowed) or ID |
| `pool-category` | Pool category (wildcards allowed) |
| `tag-count` | Number of tags |
| `comment-count` | Number of comments |
| `fav-count` | Number of favorites |
| `note-count` | Number of annotations |
| `note-text` | Note text (wildcards allowed) |
| `relation-count` | Number of relations |
| `feature-count` | Number of times featured |
| `type` | Content type: `image`, `animation`, `flash`, `video`, or `document` |
| `content-checksum` | BLAKE3 checksum |
| `flag` | Flag: `loop` or `sound` |
| `source` | Source string |
| `file-size` | File size in bytes |
| `image-width`, `width` | Image width |
| `image-height`, `height` | Image height |
| `image-area`, `area` | Pixel area |
| `image-aspect-ratio`, `image-ar`, `ar`, `aspect-ratio` | Aspect ratio |
| `creation-date`, `creation-time`, `date`, `time` | Post date |
| `last-edit-date`, `last-edit-time`, `edit-date`, `edit-time` | Last edit date |
| `comment-date`, `comment-time` | Comment date |
| `fav-date`, `fav-time` | Favorite date |
| `feature-date`, `feature-time` | Feature date |
| `safety`, `rating` | Safety: `safe`, `sketchy`, `unsafe` |

**Sort Tokens:** `random`, `id`, `score`, `uploader`/`upload`/`submit`, `pool-count`/`pool`, `tag-count`/`tag`, `comment-count`/`comment`, `fav-count`/`fav`, `note-count`, `relation-count`, `feature-count`, `type`, `flag`, `source`, `file-size`, `image-width`/`width`, `image-height`/`height`, `image-area`/`area`, `image-aspect-ratio`/`image-ar`/`ar`/`aspect-ratio`, `creation-date`/etc., `last-edit-date`/etc., `comment-date`/etc., `fav-date`/etc., `feature-date`/etc., `safety`/`rating`

**Special Tokens:** `liked`, `disliked`, `fav`, `tumbleweed`

**Successful Response:** `200` with `PagedResponse<PostInfo>`

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::PostList` privilege.
2. Cap `limit` at `MAX_POSTS_PER_PAGE` (1000).
3. Open a database transaction.
4. Build a `QueryBuilder` from the search query.
5. Set offset and limit on the query builder.
6. Execute `query_builder.list(conn)` to get `(total, selected_post_ids)`.
7. Batch-load `PostInfo` objects from the selected post IDs with the requested fields.
8. Return a `PagedResponse` as JSON.

---

### 2. `GET /post/{id}` — Get Single Post

**Handler:** `get` at `post.rs:231`

**Authentication:** Requires `Action::PostView`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low
- `403` — Post is hidden (by preferences/tag blacklist)
- `404` — Post does not exist

**Handler Logic:**
1. Verify `Action::PostView` privilege.
2. Open a database transaction.
3. Call `verify_visibility` to check the post exists and is not hidden.
4. Load `PostInfo` from the post ID with the requested fields.
5. Return as JSON.

---

### 3. `GET /post/{id}/around` — Get Neighboring Posts

**Handler:** `get_neighbors` at `post.rs:273`

**Authentication:** Requires `Action::PostList`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Response Type:** `PostNeighbors`

| Field | Type | Description |
|-------|------|-------------|
| `prev` | `Option<PostInfo>` | The previous post, or null |
| `next` | `Option<PostInfo>` | The next post, or null |

**Error Responses:**
- `403` — Privileges too low
- `403` — Post is hidden
- `404` — Post does not exist

**Handler Logic:**
1. Verify `Action::PostList` privilege.
2. Open a database transaction.
3. Call `verify_visibility` for the target post.
4. Build a `QueryBuilder` from the search query.
5. **If no sort order specified (optimized path):**
   - Query for the post with `id > post_id` ascending (previous post).
   - Query for the post with `id < post_id` descending (next post).
   - Batch-load `PostInfo` for both IDs.
   - Arrange results: if both found, first=prev, second=next; if one found and `has_previous_post`, return as prev; otherwise return as next.
6. **If sort order is specified (general path):**
   - Start with `offset = 0`, `limit = 1000`.
   - Loop: load post IDs with current offset/limit.
   - Find the position of `post_id` in the batch.
   - If batch is smaller than the limit, or limit is `i64::MAX`, or batch is `usize::MAX`, or the post is found (and not at the last position): break the loop.
   - Otherwise: increase offset by `limit - 2`, multiply limit by 10.
   - Get prev/next post IDs from positions around the found post.
   - Batch-load `PostInfo` for both IDs.
7. Return `PostNeighbors` as JSON.

---

### 4. `GET /featured-post` — Get Currently Featured Post

**Handler:** `get_featured` at `post.rs:373`

**Authentication:** Requires `Action::PostViewFeatured`.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `Option<PostInfo>` (null if no post is featured)

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::PostViewFeatured` privilege.
2. Open a database transaction.
3. Query `post_feature` table ordered by `time` descending.
4. Apply anonymous user preferences: filter out hidden posts.
5. Get the first (most recent) featured post ID.
6. If found, load `PostInfo`; otherwise return `None`.
7. Return as JSON.

---

### 5. `POST /featured-post` — Feature a Post

**Handler:** `feature` at `post.rs:421`

**Authentication:** Requires `Action::PostFeature`. User must be logged in.

**Request Body:** `FeatureBody`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | ID of the post to feature |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low
- `409` — Post is already featured (`ApiError::AlreadyExists(ResourceProperty::PostFeature)`)

**Handler Logic:**
1. Verify `Action::PostFeature` privilege.
2. Extract user ID (return `NotLoggedIn` if anonymous).
3. Open a database transaction.
4. Query the most recent featured post ID from `post_feature`.
5. If the new featured post ID equals the current one, return `ApiError::AlreadyExists`.
6. Insert a `NewPostFeature` record into `post_feature`.
7. Map foreign key violations for the post reference.
8. Record a `feature_snapshot` via the snapshot system.
9. After the first transaction commits, open a second transaction.
10. Load `PostInfo` for the featured post and return as JSON.

---

### 6. `POST /posts/reverse-search` — Reverse Image Search

**Handler:** `reverse_search` at `post.rs:565`

**Accepts:** `application/json` or `multipart/form-data`

**Authentication:** Requires `Action::PostReverseSearch`.

**Request Body:** `ReverseSearchBody`

| Field | Type | Description |
|-------|------|-------------|
| `contentToken` | `Option<UploadToken>` | Token referencing uploaded content |
| `contentUrl` | `Option<Url>` | URL to fetch content from |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Response Type:** `ReverseSearchResponse`

| Field | Type | Description |
|-------|------|-------------|
| `exactPost` | `Option<PostInfo>` | Post with the same checksum, if found |
| `similarPosts` | `Vec<SimilarPost>` | Visually similar posts |

**`SimilarPost` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `distance` | `f64` (0.0–1.0) | Visual similarity distance (lower = more similar) |
| `post` | `PostInfo` | The similar post |

**Error Responses:**
- `400` — Content is missing
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::PostReverseSearch` privilege.
2. **If JSON:** call `reverse_search_impl` directly.
3. **If multipart:** extract upload parts (Content). If a content token is present, build the body from it. If metadata is present, deserialize from JSON. Otherwise, return `ApiError::MissingFormData`.
4. **`reverse_search_impl`:**
   - Verify `Action::PostReverseSearch`.
   - Create a `Content` object from token or URL.
   - Compute content properties (checksum, signature, etc.).
   - Open a database transaction.
   - **Exact match check:** Query `post::table` for a post with the same checksum.
   - **Similarity search:** Generate signature indexes and call `PostSignature::find_similar_candidates`.
   - Filter candidates: exclude the exact match, compute Hamming distance, compare against `1.0 - config.post_similarity_threshold`.
   - Sort remaining candidates by distance.
   - Load `PostInfo` for the exact match and similar posts.
   - Return `ReverseSearchResponse`.

---

### 7. `POST /posts` — Create a Post

**Handler:** `create` at `post.rs:764`

**Accepts:** `application/json` or `multipart/form-data`

**Authentication:** Requires `Action::PostCreateIdentified` or `Action::PostCreateAnonymous` (if `anonymous: true`).

**Request Body:** `PostCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `safety` | `PostSafety` | Safety rating: `safe`, `sketchy`, `unsafe` |
| `contentToken` | `Option<UploadToken>` | Upload token for content |
| `contentUrl` | `Option<Url>` | URL to fetch content |
| `thumbnailToken` | `Option<UploadToken>` | Upload token for custom thumbnail |
| `thumbnailUrl` | `Option<Url>` | URL to fetch thumbnail |
| `source` | `Option<String>` | Source URL or description |
| `description` | `Option<String>` | Post description |
| `launchDate` | `Option<DateTime>` | Launch date |
| `mediaCreatedDate` | `Option<DateTime>` | Media creation date |
| `relations` | `Option<Vec<i64>>` | Related post IDs |
| `anonymous` | `Option<bool>` | If true, uploader name is not recorded |
| `tags` | `Option<Vec<SmallString>>` | Tags to apply |
| `notes` | `Option<Vec<Note>>` | Post annotations |
| `flags` | `Option<Vec<PostFlag>>` | Flags: `loop`, `sound` |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `400` — Content is missing
- `403` — Privileges too low
- `404` — Relations refer to non-existing posts
- `409` — Post content already exists (duplicate checksum)
- `422` — Invalid tag name, safety, notes, or flags

**Handler Logic (create_impl):**
1. Determine action: `PostCreateAnonymous` if `anonymous == Some(true)`, else `PostCreateIdentified`.
2. Verify privilege for the action.
3. Create `Content` from token or URL. Return `MissingContent` if neither is provided.
4. Compute content properties (call `get_or_compute_properties`).
5. If a custom thumbnail is provided, process it via `Content::thumbnail`.
6. Combine content flags with user-provided flags.
7. Call `tagging_update` with `tags_updated = body.tags.is_some()`.
8. **Inside the transaction:**
   - Resolve or create tag IDs using `update::tag::get_or_create_tag_ids` (`FetchMode::Deep`).
   - Insert `NewPost` into `post::table` with `ON CONFLICT (checksum) DO NOTHING`. If the row already existed, return `ApiError::AlreadyExists(ResourceProperty::PostContent)`.
   - Generate a `PostHash` for the new post ID.
   - Set tags via `update::post::set_tags`.
   - Add relations via `update::post::add_relations`.
   - Set notes via `update::post::set_notes`.
   - Insert post signature (`NewPostSignature`) into `post_signature` table.
   - Move content from temp path to permanent path via `filesystem::move_file`.
   - Generate generated thumbnail via `update::post::thumbnail(ThumbnailCategory::Generated)`.
   - If custom thumbnail exists, verify `PostEditThumbnail` privilege and create custom thumbnail.
   - Record a creation snapshot with all post data (safety, checksum, flags, source, description, dates, tags, relations, notes, custom fields).
   - Return the new post ID.
9. After committing, load `PostInfo` from the new post ID and return as JSON.

---

### 8. `POST /post-merge` — Merge Posts

**Handler:** `merge` at `post.rs:817`

**Authentication:** Requires `Action::PostMerge`.

**Request Body:** `PostMergeBody`

| Field | Type | Description |
|-------|------|-------------|
| `remove` | `i64` | Source post ID to remove |
| `mergeTo` | `i64` | Target post ID to merge into |
| `removeVersion` | `DateTime` | Version of source post |
| `mergeToVersion` | `DateTime` | Version of target post |
| `replaceContent` | `bool` | If true, replace target content with source content |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Source or target post does not exist
- `409` — Version of either post is outdated
- `422` — Source equals target (`ApiError::SelfMerge`)

**Handler Logic:**
1. Verify `Action::PostMerge` privilege.
2. If `remove == merge_to`, return `ApiError::SelfMerge`.
3. Call `tagging_update` with `tags_updated = true`.
4. **Inside the transaction:**
   - Load the source post. Return `NotFound` if absent.
   - Load the target post. Return `NotFound` if absent.
   - Verify versions of both posts.
   - Call `update::post::merge` to merge tags, relations, scores, favorites, and comments. If `replaceContent` is true, replace the target's content using the source's content.
   - Record a `merge_snapshot`.
5. After committing, load `PostInfo` for the merged-to post and return as JSON.

---

### 9. `POST /post/{id}/favorite` — Add Favorite

**Handler:** `favorite` at `post.rs:873`

**Authentication:** Requires `Action::PostFavorite`. User must be logged in.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low or not logged in
- `404` — Post does not exist

**Handler Logic:**
1. Verify `Action::PostFavorite` privilege.
2. Extract user ID (return `NotLoggedIn` if anonymous).
3. Open a database transaction.
4. Delete any existing favorite for this user+post pair (`post_favorite::table.find((post_id, user_id))`).
5. Insert a new `PostFavorite` record with the current time.
6. Map foreign key violations (post reference).
7. After committing, load `PostInfo` and return as JSON.

---

### 10. `DELETE /post/{id}/favorite` — Remove Favorite

**Handler:** `unfavorite` at `post.rs:1311`

**Authentication:** Requires `Action::PostFavorite`. User must be logged in.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low or not logged in
- `404` — Post does not exist

**Handler Logic:**
1. Verify `Action::PostFavorite` privilege.
2. Extract user ID.
3. Delete from `post_favorite::table.find((post_id, user_id))`. Return `NotFound` if the favorite did not exist.
4. Load `PostInfo` and return as JSON.

---

### 11. `PUT /post/{id}/score` — Rate a Post

**Handler:** `rate` at `post.rs:917`

**Authentication:** Requires `Action::PostScore`. User must be logged in.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Request Body:** `RatingBody`

| Field | Type | Description |
|-------|------|-------------|
| `score` | `Rating` | Score value (-1, 0, 1) |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low or not logged in
- `404` — Post does not exist
- `422` — Score is invalid

**Handler Logic:**
1. Verify `Action::PostScore` privilege.
2. Extract user ID.
3. Open a database transaction.
4. Delete any existing score for this user+post pair.
5. If the score is a valid `Score` value (-1 or 1, not 0), insert a new `PostScore` record with the current time. Score of 0 removes the rating (no insert).
6. Map foreign key violations.
7. After committing, load `PostInfo` and return as JSON.

---

### 12. `PUT /post/{id}` — Update a Post

**Handler:** `update` at `post.rs:1211`

**Accepts:** `application/json` or `multipart/form-data`

**Authentication:** Different privileges for each field: `PostEditSafety`, `PostEditFlag`, `PostEditSource`, `PostEditDescription`, `PostEditInfoDates`, `PostEditRelation`, `PostEditTag`, `PostEditNote`, `PostEditContent`, `PostEditThumbnail`, `PostEditCustomFields`.

**Request Body:** `PostUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version (required) |
| `safety` | `Option<PostSafety>` | New safety rating |
| `source` | `Option<LargeString>` | New source |
| `description` | `Option<LargeString>` | New description |
| `launchDate` | `Option<Option<DateTime>>` | Launch date (send `null` to clear) |
| `mediaCreatedDate` | `Option<Option<DateTime>>` | Media creation date (send `null` to clear) |
| `relations` | `Option<Vec<i64>>` | New relation IDs |
| `tags` | `Option<Vec<SmallString>>` | New tag list |
| `notes` | `Option<Vec<Note>>` | New annotations |
| `flags` | `Option<Vec<PostFlag>>` | New flags |
| `contentToken` | `Option<UploadToken>` | Upload token for new content |
| `contentUrl` | `Option<Url>` | URL to fetch new content |
| `thumbnailToken` | `Option<UploadToken>` | Upload token for new custom thumbnail |
| `thumbnailUrl` | `Option<Url>` | URL to fetch custom thumbnail |
| `customFields` | `Option<HashMap<SmallString, i64>>` | Custom field map (send `{}` to clear) |

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `PostInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Post does not exist
- `404` — Relations refer to non-existing posts
- `409` — Version outdated or content checksum collision
- `422` — Invalid tag name, safety, notes, or flags

**Handler Logic (update_impl):**
1. If new content provided, compute content properties.
2. If new custom thumbnail provided, process thumbnail.
3. Call `tagging_update` with `tags_updated = body.tags.is_some()`.
4. **Inside the transaction:**
   - Load the existing post. Return `NotFound` if absent.
   - Verify version matches.
   - Retrieve old snapshot data.
   - Clone the post and snapshot data as mutable.
   - For each provided field:
     - **safety:** Verify `PostEditSafety`, update `safety` on post and snapshot.
     - **flags:** Verify `PostEditFlag`, compute from slice, update post and snapshot.
     - **source:** Verify `PostEditSource`, update on post and snapshot.
     - **description:** Verify `PostEditDescription`, update on post and snapshot.
     - **launch_date:** Verify `PostEditInfoDates`, update on post and snapshot.
     - **media_created_date:** Verify `PostEditInfoDates`, update on post and snapshot.
     - **relations:** Verify `PostEditRelation`, call `update::post::set_relations`.
     - **tags:** Verify `PostEditTag`. If `config.append_tag_implications_on_post_edit` is true, use `FetchMode::Deep`; otherwise `FetchMode::Shallow`. Call `update::tag::get_or_create_tag_ids` and `update::post::set_tags`.
     - **notes:** Verify `PostEditNote`, call `update::post::set_notes`.
     - **content properties:** If new content provided:
       - Verify `PostEditContent`.
       - Update checksum in snapshot data.
       - Update all content metadata fields (file_size, width, height, type_, mime_type, checksum, checksum_md5, flags).
       - Delete old post signature, insert new one.
       - Delete old content files, move new content from temp to permanent.
       - Replace generated thumbnail.
     - **custom_thumbnail:** Verify `PostEditThumbnail`, call `update::post::thumbnail(ThumbnailCategory::Custom)`.
     - **custom_fields:** If provided:
       - Verify `PostEditCustomFields`.
       - Delete all existing `post_custom_field` rows for this post.
       - For each custom field name-value pair, look up the custom field definition (return `NotFound` if missing), insert a new `post_custom_field` row, and add to the new snapshot data.
   - Set `last_edit_time` to `DateTime::now()`.
   - Save the updated post via `save_changes`. Map unique violation (post content checksum).
   - Record a `modification_snapshot` with old and new snapshot data.
5. After committing, load `PostInfo` and return as JSON.

---

### 13. `DELETE /post/{id}` — Delete a Post

**Handler:** `delete` at `post.rs:1250`

**Authentication:** Requires `Action::PostDelete`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | `i64` | Post ID |

**Request Body:** `DeleteBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |

**Successful Response:** `200` with `()`

**Error Responses:**
- `403` — Privileges too low
- `404` — Post does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Verify `Action::PostDelete` privilege.
2. Query `post_statistics::relation_count` for the post. Return `NotFound` if absent.
3. If `relation_count > 0`, acquire the `ANTI_DEADLOCK_MUTEX` to prevent deadlocks during cascading relation deletion.
4. Open a database transaction.
5. Load the post. Return `NotFound` if absent.
6. Verify version.
7. Retrieve snapshot data for the post.
8. Record a `deletion_snapshot`.
9. Delete the post from the database.
10. After commit, if `config.delete_source_files` is true, delete the post's content files from the filesystem.
11. Return `Json(())`.

---

## Type Definitions

### `PostNeighbors`

| Field | Type | Description |
|-------|------|-------------|
| `prev` | `Option<PostInfo>` | Previous post |
| `next` | `Option<PostInfo>` | Next post |

### `FeatureBody`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Post ID to feature |

### `ReverseSearchBody`

| Field | Type | Description |
|-------|------|-------------|
| `contentToken` | `Option<UploadToken>` | Content upload token |
| `contentUrl` | `Option<Url>` | Content URL |

### `SimilarPost`

| Field | Type | Description |
|-------|------|-------------|
| `distance` | `f64` | Similarity distance (0.0–1.0) |
| `post` | `PostInfo` | The similar post |

### `ReverseSearchResponse`

| Field | Type | Description |
|-------|------|-------------|
| `exactPost` | `Option<PostInfo>` | Exact match (same checksum) |
| `similarPosts` | `Vec<SimilarPost>` | Similar posts by visual signature |

### `PostCreateBody`

All fields documented in the create endpoint above (section 7).

### `PostMergeBody`

All fields documented in the merge endpoint above (section 8).

### `PostUpdateBody`

All fields documented in the update endpoint above (section 12).
