# Update Pipeline

## Individual Files

- [mod.md](mod.md)
- [pool.md](pool.md)
- [post.md](post.md)
- [tag.md](tag.md)
- [user.md](user.md)

## Purpose

The update module handles side effects that occur after a resource is modified. The functions in this module do NOT check that the client has the required privileges. Always verify privileges before calling these functions.

## Pool Updates (`pool.rs`)

### `last_edit_time`
Updates the `last_edit_time` column of a pool to the current time.

### `set_names`
Validates that each name matches the pool regex from the configuration. Deletes all existing names for the pool, then inserts the new list from position 0.

### `set_posts`
Replaces the ordered list of posts in a pool. First queries for posts hidden due to user preferences and appends them so they are preserved. Deletes all existing pool-post associations, then inserts the new list.

### `add_posts`
Appends new posts to the end of a pool's current post list. Each new post gets an order number starting from the current post count. Handles unique/foreign key constraint violations.

### `merge`
Merges an absorbed pool into a target pool:
1. Collects posts from the absorbed pool that are not already in the target pool.
2. Appends those posts to the target pool with incremented order positions.
3. Collects all names from the absorbed pool and appends them to the target pool.
4. Deletes the absorbed pool.
5. Updates `last_edit_time` on the target pool.

## Post Updates (`post.rs`)

### `last_edit_time`
Updates the `last_edit_time` column of a post to the current time.

### `thumbnail`
Deletes the old thumbnail file, saves the new one, and updates the corresponding thumbnail size column in the database (either `generated_thumbnail_size` or `custom_thumbnail_size`).

### `set_relations`
Replaces all relations for a post:
1. Queries for hidden posts due to user preferences and appends them.
2. Deletes all old relations for the post (bidirectional).
3. Inserts the new relations.
4. Updates `last_edit_time` for all posts involved in the changed relations (symmetric difference of old and new related posts).

### `add_relations`
Inserts bidirectional post relations. Filters out self-relations (`id != post_id`). Uses `PostRelation::new_pair` to create both directions.

### `set_tags`
Deletes all existing post-tag associations and inserts new ones.

### `set_notes`
Deletes all existing post notes and inserts new ones.

### `merge`
Merges an absorbed post into a target post. Handles:
1. **Relations**: Collects all relations involving both posts, rewrites absorbed ID to target ID, deduplicates, deletes old target relations, inserts merged set.
2. **Tags**: Copies tags from absorbed post that are not already on the target.
3. **Pools**: Copies pool memberships from absorbed post to target post where not already present.
4. **Scores**: Copies scores from absorbed post.
5. **Favorites**: Copies favorites from absorbed post.
6. **Features**: Moves all feature records from absorbed post to target post.
7. **Comments**: Moves all comments from absorbed post to target post.
8. **Description**: Concatenates descriptions with a blank line separator.
9. **Content replacement** (optional): Copies the signature and indexes from absorbed to target, then swaps the files on disk. Updates all metadata (file_size, width, height, type, mime_type, checksum, flags, source, thumbnail sizes).
10. Deletes the absorbed post.
11. Deletes source files if configured.

## Tag Updates (`tag.rs`)

### FetchMode
An enum with three modes for fetching tag IDs:
- `Shallow`: Only fetch IDs directly matching the given names.
- `Deep`: Also fetch IDs of all implications, recursively.
- `Acyclic`: Same as Deep, but also checks for implication cycles.

### `last_edit_time`
Updates the `last_edit_time` column of a tag to the current time.

### `set_names`
Validates names against the tag regex, deletes all existing names, and inserts new ones from position 0.

### `set_implications`
Validates that a tag does not imply itself (returns a cyclic dependency error). Deletes all existing implications and inserts new ones.

### `set_suggestions`
Validates that a tag does not suggest itself. Deletes all existing suggestions and inserts new ones.

### `get_or_create_tag_ids`
The main function for resolving tag names to IDs:
1. Looks up existing tag IDs matching the given names.
2. If FetchMode is Deep or Acyclic, recursively collects all implied tag IDs using a `DependencyGraph` for cycle tracking.
3. If Acyclic mode, checks for cycles and returns an error if found.
4. Filters out names that already exist (case-insensitive match against existing names).
5. Creates new tags for unrecognized names (requires TagCreate privilege).
6. Inserts creation snapshots for new tags.

### DependencyGraph
A graph structure for cycle detection in tag implications:
- `nodes`: Maps each child tag ID to a set of parent tag IDs.
- `insert(implication)`: Adds an edge. Returns true if the child node is new to the graph.
- `has_cycle()`: Performs DFS-based cycle detection in O(V+E) time. Returns true if any cycle exists.

### `merge`
Merges an absorbed tag into a target tag:
1. **Implications**: Collects all implications involving both tags, rewrites absorbed ID to target ID, deduplicates, deletes old target implications, inserts merged set.
2. **Suggestions**: Same process as implications.
3. **Post usages**: Copies post-tag associations from absorbed tag to target tag where not already present.
4. **Names**: Appends absorbed tag's names to the target tag.
5. Deletes the absorbed tag.
6. Updates `last_edit_time` on the target tag.

## User Updates (`user.rs`)

### `last_login_time`
Updates the `last_login_time` column of a user to the current time.

### `last_edit_time`
Updates the `last_edit_time` column of a user to the current time.

### `avatar`
Deletes the existing custom avatar file, saves the new one, and updates the `custom_avatar_size` column in the database.
