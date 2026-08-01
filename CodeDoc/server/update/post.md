# Update — Post

**File:** `server/src/update/post.rs` (329 lines)
**Purpose:** Post tag management, relation updates, note updates, post merging (complex), thumbnail management, feature toggling.

---

## Public Functions

### `last_edit_time(conn, post_id) -> ApiResult<()>`

Updates the `last_edit_time` of the post to the current time.

```sql
UPDATE post SET last_edit_time = NOW() WHERE id = post_id
```

---

### `thumbnail(conn, post_hash, thumbnail, thumbnail_type) -> ApiResult<()>`

Updates the thumbnail for a post. Handles both generated and custom thumbnails.

**Parameters:**
- `post_hash`: `PostHash` — identifies the post and its file location
- `thumbnail`: `&DynamicImage` — the new thumbnail image
- `thumbnail_type`: `ThumbnailCategory::Generated` or `ThumbnailCategory::Custom`

**Steps:**

1. **Delete old thumbnail:** `filesystem::delete_post_thumbnail(post_hash, thumbnail_type)` — removes the existing thumbnail file from disk.
2. **Save new thumbnail:** `filesystem::save_post_thumbnail(post_hash, thumbnail, thumbnail_type)` — writes the new thumbnail to disk, returns the file size in bytes.
3. **Update database** based on thumbnail type:
   - `Generated`:
     ```sql
     UPDATE post SET generated_thumbnail_size = <size> WHERE id = post_hash.id()
     ```
   - `Custom`:
     ```sql
     UPDATE post SET custom_thumbnail_size = <size> WHERE id = post_hash.id()
     ```

**Side effects:** Filesystem write + delete, one database UPDATE.

---

### `set_relations(conn, ctx, post_id, new_related_posts) -> ApiResult<()>`

Replaces the complete set of post relations.

**Steps:**

1. **Hidden relations preservation:** If the client has hidden posts, queries for hidden relations that exist but are not visible:
   ```sql
   SELECT post_relation.child_id FROM post_relation
   WHERE parent_id = post_id AND child_id IN (<hidden posts query>)
   ```
   Appends these hidden relation IDs to `new_related_posts`.

2. **Delete old relations:** Deletes all relations involving `post_id` from BOTH directions:
   ```sql
   DELETE FROM post_relation WHERE parent_id = post_id OR child_id = post_id
   RETURNING child_id
   ```
   Collects the returned IDs into a `HashSet<i64>`.

3. **Insert new relations:** Calls `add_relations(conn, post_id, new_related_posts)`.

4. **Update timestamps on affected posts:** Computes the symmetric difference between old and new related posts (posts that were added OR removed from relations). Updates `last_edit_time` for all of them:
   ```sql
   UPDATE post SET last_edit_time = NOW() WHERE id IN (<difference post IDs>)
   ```
   This ensures that related posts' edit times are updated when their relation set changes, even though they were not directly edited.

**Key detail:** The symmetric difference means:
- If post A was related and is still related → NO timestamp update
- If post A was related but is now removed → timestamp update
- If post A was not related but is now added → timestamp update

---

### `add_relations(conn, post_id, new_related_posts) -> ApiResult<()>`

Inserts bidirectional relations between `post_id` and each post in `new_related_posts`.

**Steps:**

1. Filters out `post_id` itself (a post cannot relate to itself).
2. For each other post ID, creates a pair of `PostRelation` entries via `PostRelation::new_pair(post_id, other_id)`:
   - `(parent: post_id, child: other_id)`
   - `(parent: other_id, child: post_id)`
3. Batch inserts all relation pairs:
   ```sql
   INSERT INTO post_relation (parent_id, child_id) VALUES ...
   ```
4. Maps errors:
   - Unique violation → `ApiError::ResourceAlreadyExists` for `ResourceProperty::PostRelation`
   - Foreign key violation → `ApiError::ResourceDoesNotExist` for `ResourceType::Post`

---

### `set_tags(conn, post_id, tags) -> QueryResult<()>`

Replaces the complete set of tags on a post.

**Parameters:**
- `tags`: `&[i64]` — list of tag IDs (not names)

**Steps:**

1. **Delete all existing tags:**
   ```sql
   DELETE FROM post_tag WHERE post_id = post_id
   ```

2. **Insert new tags:**
   ```sql
   INSERT INTO post_tag (post_id, tag_id) VALUES ...
   ```

**Note:** Returns `QueryResult<()>`, not `ApiResult`. No error mapping is done here — the caller must handle constraint violations.

---

### `set_notes(conn, post_id, notes) -> QueryResult<()>`

Replaces the complete set of notes (annotations) on a post.

**Parameters:**
- `notes`: `&[Note]` — list of note objects

**Steps:**

1. Converts each `Note` to a `NewPostNote` via `note.to_new_post_note(post_id)`.
2. **Deletes all existing notes:**
   ```sql
   DELETE FROM post_note WHERE post_id = post_id
   ```
3. **Inserts new notes:**
   ```sql
   INSERT INTO post_note (...) VALUES ...
   ```

Returns `QueryResult<()>` — no error mapping.

---

### `merge(conn, config, absorbed_post, merge_to_post, replace_content) -> ApiResult<()>`

**The most complex function in the update module.** Merges one post into another. All data from the absorbed post is transferred to the target post.

**Parameters:**
- `absorbed_post`: `&Post` — the post being absorbed/removed
- `merge_to_post`: `&Post` — the post receiving all data
- `replace_content`: `bool` — if true, the absorbed post's content (file, checksum, dimensions, flags, source) replaces the target's content

**Steps:**

1. **Initialize:** Extract IDs and compute `PostHash` values for both posts.

2. **Merge relations** (lines 143-167):
   - Fetches all relations involving either post:
     ```sql
     SELECT * FROM post_relation
     WHERE parent_id IN (absorbed_id, merge_to_id)
     OR child_id IN (absorbed_id, merge_to_id)
     ```
   - For each relation, replaces `absorbed_id` with `merge_to_id` (the surviving post ID).
   - Filters out self-references (where `parent_id == child_id`).
   - Uses `HashSet` to deduplicate relations.
   - Deletes all old relations for `merge_to_id`:
     ```sql
     DELETE FROM post_relation WHERE parent_id = merge_to_id OR child_id = merge_to_id
     ```
   - Batch inserts the merged, deduplicated relations.

3. **Merge tags** (lines 170-185):
   - Queries tags of the target post and absorbed post.
   - Filters out tags already on the target:
     ```sql
     SELECT tag_id FROM post_tag
     WHERE post_id = absorbed_id
     AND tag_id NOT IN (SELECT tag_id FROM post_tag WHERE post_id = merge_to_id)
     ```
   - Inserts new tags as `PostTag{post_id: merge_to_id, tag_id}`.

4. **Merge pools** (lines 188-204):
   - Same pattern as tags: finds pool memberships of the absorbed post not already on the target.
   - Preserves the `order` field from the original pool membership.
   - Inserts as `PoolPost{pool_id, post_id: merge_to_id, order}`.

5. **Merge scores** (lines 207-224):
   - Merges user scores. Filters out users who already scored the target post.
   - Preserves the `score` value and `time` from the absorbed post.
   - Inserts as `PostScore{post_id: merge_to_id, user_id, score, time}`.

6. **Merge favorites** (lines 227-243):
   - Same pattern: finds unique favorites (users who favorited the absorbed post but not the target).
   - Preserves `time` of the favorite action.
   - Inserts as `PostFavorite{post_id: merge_to_id, user_id, time}`.

7. **Merge features** (lines 246-256):
   - **Deletes** all feature records from the absorbed post and returns them:
     ```sql
     DELETE FROM post_feature WHERE post_id = absorbed_id RETURNING user_id, time
     ```
   - Re-inserts them pointing to `merge_to_id` as `NewPostFeature{post_id: merge_to_id, user_id, time}`.
   - Uses DELETE + RETURNING instead of SELECT + INSERT, which moves the records atomically.

8. **Merge comments** (lines 259-271):
   - **Deletes** all comments from the absorbed post and returns the data:
     ```sql
     DELETE FROM comment WHERE post_id = absorbed_id RETURNING user_id, text, creation_time
     ```
   - Re-inserts as `NewComment{user_id, post_id: merge_to_id, text, creation_time}`.

9. **Merge descriptions** (lines 273-278):
   - Concatenates descriptions: `merge_to_post.description + "\n\n" + absorbed_post.description`
   - Trims whitespace.
   - Updates the target post's description in the database.

10. **Content replacement** (lines 280-288, conditional on `replace_content`):
    - Fetches the signature and word indexes from the absorbed post:
      ```sql
      SELECT signature, words FROM post_signature WHERE id = absorbed_id
      ```
    - Updates the target post's signature:
      ```sql
      UPDATE post_signature SET signature = <abs>, words = <abs> WHERE id = merge_to_id
      ```
    - This must be done BEFORE deleting the absorbed post because the `post_signature` table has a CASCADE delete foreign key.

11. **Delete absorbed post** (line 290):
    ```sql
    DELETE FROM post WHERE id = absorbed_id
    ```

12. **File swap** (lines 292-299, conditional on `replace_content`):
    - Calls `filesystem::swap_posts(config, &absorbed_hash, absorbed_mime, &merge_to_hash, merge_to_mime)` — swaps the actual files on disk.

13. **Update metadata** (lines 302-316, conditional on `replace_content`):
    - Updates the target post with the absorbed post's metadata:
      ```sql
      UPDATE post SET
        file_size, width, height, type_, mime_type,
        checksum, checksum_md5, flags, source,
        generated_thumbnail_size, custom_thumbnail_size
        = <absorbed values>
      WHERE id = merge_to_id
      ```
    - This must be done AFTER deletion because `checksum` has a UNIQUE constraint.

14. **Delete source files** (lines 319-326, conditional on `config.delete_source_files`):
    - Determines which content type to delete: if `replace_content`, delete the merge target's old file; otherwise delete the absorbed post's file.
    - Calls `filesystem::delete_post(&absorbed_hash, deleted_content_type)`.

15. **Update timestamp** (line 327):
    - Calls `last_edit_time(conn, merge_to_id)`.

---

## Side Effects Summary

| Function | Database Writes | Database Reads | Filesystem |
|---|---|---|---|
| `last_edit_time` | UPDATE post | — | — |
| `thumbnail` | UPDATE post | — | Delete + Save thumbnail file |
| `set_relations` | DELETE + INSERT post_relation, UPDATE post | SELECT post_relation (hidden) | — |
| `add_relations` | INSERT post_relation | — | — |
| `set_tags` | DELETE + INSERT post_tag | — | — |
| `set_notes` | DELETE + INSERT post_note | — | — |
| `merge` | Many INSERT + DELETE + UPDATE (see steps) | Many SELECT | swap_posts, delete_post |

The `merge` function is the most impactful, touching 9+ database tables and potentially the filesystem.
