# Update — Tag

**File:** `server/src/update/tag.rs` (417 lines)
**Purpose:** Tag implication chain resolution, tag auto-creation, tag merging, name/suggestion/implication management, cycle detection. **This is critical business logic.**

---

## Enum: `FetchMode`

Controls how tag IDs are resolved from tag names in `get_or_create_tag_ids`.

| Variant | Behavior |
|---|---|
| `Shallow` | Only fetch IDs whose names match the input names exactly |
| `Deep` | Fetch IDs of matching names AND all their implications, recursively, until no new tags are found |
| `Acyclic` | Same as `Deep`, but also runs cycle detection on the implication graph. Returns `ApiError::CyclicDependency` if a cycle exists. |

---

## Public Functions

### `last_edit_time(conn, tag_id) -> ApiResult<()>`

Updates the `last_edit_time` of the tag to the current time.

```sql
UPDATE tag SET last_edit_time = NOW() WHERE id = tag_id
```

---

### `set_names(conn, config, tag_id, names) -> ApiResult<()>`

Replaces the complete ordered list of names for a tag.

**Steps:**

1. Validates each name against the `RegexType::Tag` regex from `config`.
2. **Deletes all existing names:**
   ```sql
   DELETE FROM tag_name WHERE tag_id = tag_id
   ```
3. Calls `add_names(conn, tag_id, 0, names)` to insert new names from order 0.

---

### `set_implications(conn, tag_id, implied_ids) -> ApiResult<()>`

Replaces the complete set of implications for a tag. An implication means: "when this tag is added to a post, also add all implied tags."

**Steps:**

1. Validates that no implication points to itself — returns `ApiError::CyclicDependency(ResourceType::TagImplication)` if `tag_id == child_id`.
2. **Deletes all existing implications:**
   ```sql
   DELETE FROM tag_implication WHERE parent_id = tag_id
   ```
3. **Inserts new implications:**
   ```sql
   INSERT INTO tag_implication (parent_id, child_id) VALUES ...
   ```

**Note:** This function only checks for self-reference cycles (tag implies itself). It does NOT check for transitive cycles (A → B → C → A). Transitive cycle detection happens in `get_or_create_tag_ids` with `FetchMode::Acyclic` and in `merge` before any table write.

---

### `set_suggestions(conn, tag_id, suggested_ids) -> ApiResult<()>`

Replaces the complete set of suggestions for a tag. A suggestion means: "when this tag is added, prompt the user to also add these suggested tags" (optional, not automatic like implications).

**Steps:**

1. Validates no self-suggestion — returns `ApiError::CyclicDependency(ResourceType::TagSuggestion)` if `tag_id == child_id`.
2. **Deletes all existing suggestions:**
   ```sql
   DELETE FROM tag_suggestion WHERE parent_id = tag_id
   ```
3. **Inserts new suggestions:**
   ```sql
   INSERT INTO tag_suggestion (parent_id, child_id) VALUES ...
   ```

---

### `get_or_create_tag_ids(conn, ctx, names, mode) -> ApiResult<(Vec<i64>, Vec<SmallString>)>`

**The critical business logic function.** Resolves tag names to tag IDs, following implication chains as directed by `FetchMode`. Auto-creates tags for names that do not exist.

**Parameters:**
- `names`: `Vec<SmallString>` — the raw tag names to resolve
- `mode`: `FetchMode` — controls how deeply to follow implications

**Returns:** Tuple of `(tag_ids, primary_tag_names)` — all resolved tag IDs and their primary names.

---

### Code Paths — Step by Step

#### Step 1: Initial Name Matching (line 98-102)

```sql
SELECT DISTINCT tag_id FROM tag_name WHERE name IN (<names>)
```

Finds all tag IDs whose ANY name (primary or alias) matches one of the input names. Uses `DISTINCT` because a tag can have multiple names.

#### Step 2: Build Dependency Graph (lines 105-128)

A `DependencyGraph` is initialized with the found tag IDs.

**If mode is `Shallow`:** Skip to Step 3.

**If mode is `Deep` or `Acyclic`:** Enter a loop:

```sql
SELECT DISTINCT * FROM tag_implication WHERE parent_id IN (<current IDs>)
```

Each iteration:
1. Fetches all implications where the parent is in the current set of tag IDs.
2. For each fetched implication `(parent_id, child_id)`:
   - Checks if `child_id` is already in the graph.
   - If NOT in the graph: inserts the implication (adds the child node with edges from parents) and adds `child_id` to `implied_ids` for the next iteration.
   - If already in the graph: skips (preventing infinite loops on duplicate edges).
3. If no new nodes were added (`dependency_graph.len() == previous_len`), the loop terminates.

This implements **recursive implication expansion**: if tag A implies tag B, and tag B implies tag C, then resolving A returns A, B, and C.

**Cycle detection (Acyclic mode only, line 125-127):**
After the expansion loop, calls `dependency_graph.has_cycle()`. If a cycle exists, returns `ApiError::CyclicDependency(ResourceType::TagImplication)`.

#### Step 3: Collect All Tag IDs (line 130)

```rust
let tag_ids: Vec<_> = dependency_graph.into_nodes().collect();
```

Collects all nodes in the dependency graph. This includes:
- Tags directly matched by name
- All transitively implied tags (if Deep/Acyclic mode)

#### Step 4: Identify New Names (lines 131-142)

```sql
SELECT name FROM tag_name WHERE tag_id IN (<all tag IDs>)
```

Fetches all existing names for all resolved tag IDs. Converts to lowercase for case-insensitive comparison.

Filters the input `names` to find which ones do not match any existing name (case-insensitive):
```rust
names.into_iter()
    .filter(|name| !existing_names.contains(&name.to_lowercase()))
    .collect()
```

#### Step 5: Auto-Create New Tags (lines 143-164, conditional)

Only executes if `new_names` is non-empty.

**Validation:**
- Checks each new name against `RegexType::Tag` regex.
- Verifies `Action::TagCreate` privilege via `ctx.verify_privilege(...)`.

**Database operations:**

1. **Insert tag row(s):**
   ```sql
   INSERT INTO tag DEFAULT VALUES RETURNING id
   ```
   Creates one new tag per new name using `NewTag::default()`. Returns the new IDs.

2. **Insert tag names:**
   ```sql
   INSERT INTO tag_name (tag_id, order, name) VALUES ...
   ```
   Each new tag gets one name with `order = 0` (primary name).

3. **Create snapshots:**
   Calls `snapshot::tag::new_name_snapshots(conn, client, new_names)` to record snapshots for the auto-created tags.

4. **Extend tag IDs:**
   Appends the new tag IDs to `tag_ids`: `tag_ids.extend(new_tag_ids)`.

#### Step 6: Fetch Primary Names (lines 166-170)

```sql
SELECT name FROM tag_name WHERE tag_id IN (<all tag IDs>) AND order = 0
```

Fetches the primary name (order 0) for each resolved tag ID.

#### Step 7: Return (line 171)

```rust
Ok((tag_ids, primary_tag_names))
```

---

### `merge(conn, absorbed_id, merge_to_id) -> ApiResult<()>`

Merges one tag into another. All data from the absorbed tag transfers to the target tag. Rewriting edges during a merge can close cycles that create/update reject (they build the graph with `FetchMode::Acyclic`), so `merge` detects cycles before any table write.

**Steps:**

1. **Merge implications (prepare):**
   - Fetches all implications involving either tag:
     ```sql
     SELECT * FROM tag_implication
     WHERE parent_id IN (absorbed_id, merge_to_id)
     OR child_id IN (absorbed_id, merge_to_id)
     ```
   - Remaps `absorbed_id` to `merge_to_id` in all fetched implications.
   - Filters out self-implications (`parent_id == child_id`).
   - Uses `HashSet` to deduplicate.

2. **Detect implication cycles (before any table write):**
   - Creates a `DependencyGraph` seeded with `merge_to_id` and every merged edge. Each merged edge touches `merge_to_id`, so any new cycle through the target tag is reachable.
   - Builds the initial frontier from the parent and child IDs of all merged implications.
   - Expands transitively over the live `tag_implication` table, exactly like `get_or_create_tag_ids` does:
     - Loads all implications whose parent is in the frontier:
       ```sql
       SELECT DISTINCT * FROM tag_implication WHERE parent_id IN (<frontier>)
       ```
     - Inserts each implication into the graph.
     - Collects the new children as the next frontier.
     - Repeats until no new node appears.
   - If `dependency_graph.has_cycle()` returns true, returns `ApiError::CyclicDependency(ResourceType::TagImplication)` (422). No table is written.

3. **Write merged implications:**
   - Deletes old implications from `merge_to_id`:
     ```sql
     DELETE FROM tag_implication WHERE parent_id = merge_to_id OR child_id = merge_to_id
     ```
   - Inserts the deduplicated, remapped implications.

4. **Merge suggestions:**
   - Same pattern as implications but on `tag_suggestion` table.
   - Remaps, deduplicates, deletes old, inserts new.

5. **Merge usages (post tags):**
   - Finds posts tagged with the absorbed tag but NOT the target:
     ```sql
     SELECT post_id FROM post_tag
     WHERE tag_id = absorbed_id
     AND post_id NOT IN (SELECT post_id FROM post_tag WHERE tag_id = merge_to_id)
     ```
   - Creates new `PostTag{post_id, tag_id: merge_to_id}` entries for those posts.

6. **Merge names:**
   - Gets next order number:
     ```sql
     SELECT MAX(order) + 1 FROM tag_name WHERE tag_id = merge_to_id
     ```
   - Deletes and returns names from absorbed tag:
     ```sql
     DELETE FROM tag_name WHERE tag_id = absorbed_id RETURNING name
     ```
   - Appends names to target via `add_names(conn, merge_to_id, current_name_count, &removed_names)`.

7. **Delete absorbed tag:**
   ```sql
   DELETE FROM tag WHERE id = absorbed_id
   ```

8. **Update timestamp:**
   Calls `last_edit_time(conn, merge_to_id)`.

---

## Struct: `DependencyGraph`

A directed graph data structure for cycle detection in tag implication chains. Maps each tag (node) to its parents (incoming edges).

### Fields

| Field | Type | Description |
|---|---|---|
| `nodes` | `HashMap<i64, HashSet<i64>>` | Maps child_id → set of parent_ids |

The graph may consist of multiple disjoint subgraphs. Each node maps to its set of parent nodes (the tags that imply it).

### Method: `new(starting_nodes) -> Self`

Creates a graph with the given starting nodes and empty edge sets (no connections). Each starting node maps to an empty `HashSet`.

### Method: `len() -> usize`

Returns the total number of nodes in the graph.

### Method: `insert(implication) -> bool`

Inserts an implication edge into the graph. Returns `true` if the child node was newly created (was not previously in the graph), `false` if it already existed.

**Algorithm:**
1. Gets or creates the entry for `implication.child_id`.
2. Records whether the entry was new (`parents.is_empty()` — an empty set means a fresh `or_default()`).
3. Inserts `implication.parent_id` into the parents set.
4. Returns whether the node was new.

### Method: `into_nodes() -> IntoKeys<i64, HashSet<i64>>`

Consumes the graph and returns an iterator over all node keys (tag IDs). This is how the resolved tag IDs are extracted after the implication expansion loop.

### Method: `has_cycle() -> bool`

Performs **depth-first search (DFS) cycle detection**. Runs in O(V + E) time.

**Algorithm:**

1. Initializes `traversed_nodes: HashMap<i64, TraversalState>` — tracks each node's traversal state.
2. For each node `(base_node_id, parents)` in `nodes`:
   - Initializes a `traversal_stack: Vec<(i64, Iter)>` — a stack of `(node_id, iterator_over_parents)` pairs.
   - While the stack is not empty:
     a. Peeks at the top of the stack.
     b. Gets the next parent from the iterator.
     c. If no more parents: marks the current node as `Explored`, pops the stack, continues.
     d. Checks the traversal state of the next node:
        - `Visited`: A cycle exists (we found a back edge). **Returns true.**
        - `Explored`: Already processed. Skip.
        - Not in map: Insert as `Visited` and push to stack (continue DFS).
     e. If the next node has parents in `nodes`, push it to the stack with its parent iterator.

3. If all nodes processed without finding a back edge: **returns false.**

**State semantics:**
- `Visited`: The node is currently on the DFS stack (part of the current exploration path). Finding a `Visited` node means a back edge → cycle.
- `Explored`: The node and all its descendants have been fully processed. Finding an `Explored` node means no cycle from this node.

---

## Enum: `TraversalState`

| Variant | Meaning |
|---|---|
| `Visited` | Node is on the current DFS stack — part of the active exploration path |
| `Explored` | Node and all descendants have been fully processed |

---

## Private Function: `add_names(conn, tag_id, current_name_count, names) -> ApiResult<()>`

Appends names to a tag's name list.

**Parameters:**
- `current_name_count`: current number of names (type `i32`, used for order)

**Steps:**

1. Computes `total_name_count`: `min(current_name_count + names.len(), i32::MAX)`.
2. Creates `NewTagName{tag_id, order, name}` entries with sequential orders.
3. Batch inserts:
   ```sql
   INSERT INTO tag_name (tag_id, order, name) VALUES ...
   ```
4. Maps unique violations (`ResourceProperty::TagName`) to `ApiError::ResourceAlreadyExists`.

---

## Tests

### `trivial_graphs`

Tests basic cycle detection cases:
- Empty graph → no cycle
- Single node 0 → no cycle
- Self-loop (0 → 0) → cycle detected
- Two nodes with one edge (1 → 0) → no cycle
- Two nodes with edges in both directions (1 → 0 and 0 → 1) → cycle detected

### `dependency_graph_with_cycle`

Tests a multi-node graph that contains a 2-node cycle between nodes 3 and 4 (4 → 3 via the edge `(3, 4)`, and 3 → 4 via path 1→3, 3→4, 4→3). Verifies cycle is detected.

### `dependency_graph_without_cycle`

Tests a 12-node DAG (Directed Acyclic Graph) with edges:
```
0→1, 0→2, 0→3, 2→5, 3→4, 3→9, 4→5, 5→6, 5→7, 6→8, 7→8, 8→9, 9→10
```
Verifies no cycle is detected in this valid DAG.

---

## Database Query Summary

| Function | Writes | Reads |
|---|---|---|
| `last_edit_time` | UPDATE tag | — |
| `set_names` | DELETE + INSERT tag_name | — |
| `set_implications` | DELETE + INSERT tag_implication | — |
| `set_suggestions` | DELETE + INSERT tag_suggestion | — |
| `get_or_create_tag_ids` | INSERT tag (conditionally), INSERT tag_name (conditionally) | Many SELECT from tag_name, tag_implication |
| `merge` | Multiple INSERT + DELETE | Multiple SELECT from tag_implication, tag_suggestion, post_tag, tag_name |
| `add_names` | INSERT tag_name | — |

---

## Side Effects

- `get_or_create_tag_ids` may auto-create tags, insert names, and create snapshots
- `merge` touches tag_implication, tag_suggestion, post_tag, tag_name, and tag tables
- `merge` rejects a cyclic merge before any table write and leaves the database untouched
- All functions operate within a database transaction context
- No filesystem operations in this module
- `get_or_create_tag_ids` calls `ctx.verify_privilege(Action::TagCreate)` when creating tags
- `get_or_create_tag_ids` creates snapshots via `snapshot::tag::new_name_snapshots` for auto-created tags
