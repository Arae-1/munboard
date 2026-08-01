# Update — Pool

**File:** `server/src/update/pool.rs` (118 lines)
**Purpose:** Pool post count recalculation, name/posts list management, pool merging.

---

## Public Functions

### `last_edit_time(conn, pool_id) -> QueryResult<()>`

Updates the `last_edit_time` of the pool to the current time.

```sql
UPDATE pool SET last_edit_time = NOW() WHERE id = pool_id
```

Called as a side effect whenever a pool is modified. Does not verify privileges.

---

### `set_names(conn, config, pool_id, names) -> ApiResult<()>`

Replaces the complete list of names for a pool.

**Steps:**

1. Validates each name against the `RegexType::Pool` regex from `config`. Returns error if any name does not match.
2. **Deletes all existing names:**
   ```sql
   DELETE FROM pool_name WHERE pool_id = pool_id
   ```
3. Calls `add_names(conn, pool_id, 0, names)` to insert the new names starting from order 0.

**Important:** This is a full replacement, not an append. All old names are deleted first.

---

### `set_posts(conn, ctx, pool_id, posts) -> ApiResult<()>`

Replaces the complete ordered list of posts in a pool.

**Steps:**

1. **Hidden posts preservation:** If the client has hidden posts (via `preferences::hidden_posts`), queries for hidden posts that exist in the pool but are not visible to the client:
   ```sql
   SELECT pool_post.post_id FROM pool_post
   WHERE pool_id = pool_id AND post_id IN (<hidden posts query>)
   ```
   These hidden posts are appended to `posts` so they are not accidentally removed.

2. **Deletes all existing pool-post associations:**
   ```sql
   DELETE FROM pool_post WHERE pool_id = pool_id
   ```

3. Calls `add_posts(conn, pool_id, 0, posts)` to insert the new post list starting from order 0.

**Important:** This is a full replacement. The hidden posts preservation keeps the client's view of the pool consistent even when they cannot see all posts.

---

### `add_posts(conn, pool_id, current_post_count, posts) -> ApiResult<()>`

Appends posts to the pool's existing post list.

**Parameters:**
- `current_post_count`: the number of posts already in the pool (used for order numbering)

**Steps:**

1. Computes `total_post_count` by adding `current_post_count` to `posts.len()`, saturating at `i64::MAX`.
2. Creates `PoolPost` structs for each post with sequential `order` values starting from `current_post_count`.
3. Batch inserts all `PoolPost` records:
   ```sql
   INSERT INTO pool_post (pool_id, post_id, order) VALUES ...
   ```
4. Maps unique constraint violations (`ResourceProperty::PoolPost`) or foreign key violations (`ResourceType::Post`) to `ApiError`.

**Error mapping:** Uses `error::map_unique_or_foreign_key_violation` which converts Diesel errors to:
- `ApiError::ResourceAlreadyExists` for duplicate pool-post pairs
- `ApiError::ResourceDoesNotExist` for posts that do not exist

---

### `merge(conn, absorbed_id, merge_to_id) -> ApiResult<()>`

Merges one pool into another. All posts and names from the absorbed pool are transferred to the target pool.

**Steps:**

1. **Merge posts:**
   - Queries all post IDs in both pools
   - Filters out posts already in the merge target:
     ```sql
     SELECT pool_post.post_id FROM pool_post
     WHERE pool_id = absorbed_id
     AND post_id NOT IN (SELECT post_id FROM pool_post WHERE pool_id = merge_to_id)
     ORDER BY order
     ```
   - Counts current posts in the merge target
   - Calls `add_posts(conn, merge_to_id, post_count, &new_pool_posts)` to append

2. **Merge names:**
   - Gets the next available order number for names in the merge target:
     ```sql
     SELECT MAX(order) + 1 FROM pool_name WHERE pool_id = merge_to_id
     ```
     Defaults to 0 if no names exist.
   - Deletes and returns names from the absorbed pool:
     ```sql
     DELETE FROM pool_name WHERE pool_id = absorbed_id RETURNING name
     ```
   - Calls `add_names(conn, merge_to_id, current_name_count, &removed_names)` to append

3. **Delete absorbed pool:**
   ```sql
   DELETE FROM pool WHERE id = absorbed_id
   ```

4. **Update timestamp:**
   Calls `last_edit_time(conn, merge_to_id)` on the target pool.

---

## Private Function: `add_names(conn, pool_id, current_name_count, names) -> ApiResult<()>`

Appends names to the pool's existing name list.

**Parameters:**
- `current_name_count`: current number of names (for order numbering, type `i32`)

**Steps:**

1. Computes `total_name_count` by adding `current_name_count` to `names.len()`, saturating at `i32::MAX`.
2. Creates `NewPoolName` structs with sequential `order` values starting from `current_name_count`.
3. Batch inserts all names:
   ```sql
   INSERT INTO pool_name (pool_id, order, name) VALUES ...
   ```
4. Maps unique constraint violations (`ResourceProperty::PoolName`) to `ApiError::ResourceAlreadyExists`.

**Note:** Unlike posts, names use `i32` for order counting (posts use `i64`).

---

## Side Effects Summary

| Function | Writes | Reads | Deletes |
|---|---|---|---|
| `last_edit_time` | UPDATE pool | — | — |
| `set_names` | INSERT pool_name | — | DELETE pool_name (all) |
| `set_posts` | INSERT pool_post | SELECT pool_post (hidden) | DELETE pool_post (all) |
| `add_posts` | INSERT pool_post | — | — |
| `merge` | INSERT pool_post + pool_name, DELETE pool | SELECT pool_post + pool_name | DELETE pool |
| `add_names` | INSERT pool_name | — | — |

No filesystem operations.
