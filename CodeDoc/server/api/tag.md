# Tag API — `tag.rs`

File: `server/src/api/tag.rs`

## Overview

This module defines all tag-related HTTP endpoints. It handles CRUD operations, search, sibling listing, and tag merging.

---

## Router Registration

### `routes() -> OpenApiRouter<AppState>`

**Location:** `tag.rs:28`

Returns a router with these endpoints:

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/tags` | `list` |
| `GET` | `/tag/{name}` | `get` |
| `GET` | `/tag-siblings/{name}` | `get_siblings` |
| `POST` | `/tags` | `create` |
| `POST` | `/tag-merge` | `merge` |
| `PUT` | `/tag/{name}` | `update` |
| `DELETE` | `/tag/{name}` | `delete` |

---

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_TAGS_PER_PAGE` | `1000` | Maximum tags per page in list queries |
| `MAX_TAG_SIBLINGS` | `50` | Maximum sibling tags returned |

---

## Helper: `verify_visibility`

**Location:** `tag.rs:556`

```rust
fn verify_visibility(conn, ctx, tag_name) -> ApiResult<i64>
```

1. Joins `tag`, `tag_name`, and `tag_category` tables.
2. Filters by the given tag name.
3. Returns `ApiError::NotFound(ResourceType::Tag)` if not found.
4. For anonymous users, checks if the tag name or its category is in `config.anonymous_preferences.tag_blacklist` or `tag_category_blacklist`.
5. If blacklisted, returns `ApiError::Hidden(ResourceType::Tag)`.
6. Returns the `tag_id` on success.

---

## Endpoints

### 1. `GET /tags` — List Tags

**Handler:** `list` at `tag.rs:87`

**Authentication:** Requires `Action::TagList`.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |
| `offset` | `Option<i64>` | Starting position (default: 0) |
| `limit` | `NonZeroI64` | Maximum results (capped at 1000) |

**Search Tokens (Named):**

| Token | Description |
|-------|-------------|
| `name` | Tag name (wildcards allowed) |
| `category` | Tag category (wildcards allowed) |
| `description` | Tag description (wildcards allowed) |
| `creation-date`, `creation-time` | Created at date |
| `last-edit-date`, `last-edit-time`, `edit-date`, `edit-time` | Last edit date |
| `usages`, `usage-count`, `post-count` | Number of posts using tag |
| `suggestion-count` | Number of suggestions |
| `implication-count` | Number of implications |
| `implies` | Has implication with name (wildcards allowed) |
| `suggests` | Has suggestion with name (wildcards allowed) |

**Sort Tokens:** `random`, `name`, `category`, `description`, `creation-date`/`creation-time`, `last-edit-date`/`last-edit-time`/`edit-date`/`edit-time`, `usages`/`usage-count`/`post-count`, `implication-count`/`implies`, `suggestion-count`/`suggests`

**Successful Response:** `200` with `PagedResponse<TagInfo>`

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::TagList` privilege.
2. Cap `limit` at `MAX_TAGS_PER_PAGE` (1000).
3. Open a database transaction.
4. Build `QueryBuilder` from the search query.
5. Set offset and limit.
6. Execute `query_builder.list(conn)` to get `(total, selected_tag_ids)`.
7. Batch-load `TagInfo` objects from the selected tag IDs.
8. Return `PagedResponse` as JSON.

---

### 2. `GET /tag/{name}` — Get Single Tag

**Handler:** `get` at `tag.rs:129`

**Authentication:** Requires `Action::TagView`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Tag name |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `TagInfo`

**Error Responses:**
- `403` — Privileges too low
- `403` — Tag is hidden (blacklisted for anonymous)
- `404` — Tag does not exist

**Handler Logic:**
1. Verify `Action::TagView` privilege.
2. Open a database transaction.
3. Call `verify_visibility` to get the tag ID (checks existence and anonymity preferences).
4. Load `TagInfo` from the tag ID with requested fields.
5. Return as JSON.

---

### 3. `GET /tag-siblings/{name}` — Get Tag Siblings

**Handler:** `get_siblings` at `tag.rs:183`

**Authentication:** Requires `Action::TagView`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Tag name |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Response Type:** `TagSiblings`

| Field | Type | Description |
|-------|------|-------------|
| `results` | `Vec<TagSibling>` | Sibling tags sorted by occurrence count |

**`TagSibling` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `tag` | `TagInfo` | The sibling tag |
| `occurrences` | `i64` | Number of posts where both tags appear |

**Successful Response:** `200` with `TagSiblings`

**Error Responses:**
- `403` — Privileges too low
- `403` — Tag is hidden
- `404` — Tag does not exist

**Handler Logic:**
1. Verify `Action::TagView` privilege.
2. Open a database transaction.
3. Call `verify_visibility` to get the tag ID.
4. Get all post IDs that have this tag from `post_tag`.
5. Query `post_tag` for other tags on those posts, group by `tag_id`, count occurrences, exclude the original tag ID, order by count descending, limit to `MAX_TAG_SIBLINGS` (50).
6. For anonymous users: filter out tags in the anonymous tag/category blacklist.
7. Load `TagInfo` for the sibling IDs.
8. Zip with occurrence counts to build `TagSibling` objects.
9. Return `TagSiblings` as JSON.

---

### 4. `POST /tags` — Create a Tag

**Handler:** `create` at `tag.rs:270`

**Authentication:** Requires `Action::TagCreate`.

**Request Body:** `TagCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `category` | `SmallString` | Category name (must exist) |
| `description` | `Option<LargeString>` | Tag description |
| `names` | `Vec<SmallString>` | Tag names (must match `tag_name_regex`) |
| `implications` | `Option<Vec<SmallString>>` | Implied tags (created if absent) |
| `suggestions` | `Option<Vec<SmallString>>` | Suggested tags (created if absent) |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `TagInfo`

**Error Responses:**
- `403` — Privileges too low
- `409` — Any name is already used by an existing tag
- `422` — No names given (`ApiError::NoNamesGiven`)
- `422` — Category missing/invalid
- `422` — Any name/implication/suggestion is invalid
- `422` — Implications or suggestions create a cyclic dependency

**Handler Logic:**
1. Verify `Action::TagCreate` privilege.
2. If `body.names` is empty, return `ApiError::NoNamesGiven`.
3. Open a database transaction.
4. Look up the category by name in `tag_category::table`. Return `NotFound` if absent.
5. Insert `NewTag` into `tag::table`.
6. Set names via `update::tag::set_names`.
7. Resolve/create implied tags via `update::tag::get_or_create_tag_ids` with `FetchMode::Acyclic` (prevents cycles).
8. Resolve/create suggested tags via `update::tag::get_or_create_tag_ids` with `FetchMode::Acyclic`.
9. Set implications via `update::tag::set_implications`.
10. Set suggestions via `update::tag::set_suggestions`.
11. Record a `creation_snapshot` with tag data (description, category, names, implications, suggestions).
12. After the first transaction, load `TagInfo` and return as JSON.

---

### 5. `POST /tag-merge` — Merge Tags

**Handler:** `merge` at `tag.rs:349`

**Authentication:** Requires `Action::TagMerge`.

**Request Body:** `MergeBody<SmallString>`

| Field | Type | Description |
|-------|------|-------------|
| `remove` | `SmallString` | Source tag name to remove |
| `mergeTo` | `SmallString` | Target tag name to merge into |
| `removeVersion` | `DateTime` | Version of source tag |
| `mergeToVersion` | `DateTime` | Version of target tag |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `TagInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Source or target tag does not exist
- `409` — Version outdated
- `422` — Source equals target (`ApiError::SelfMerge`)

**Handler Logic:**
1. Verify `Action::TagMerge` privilege.
2. Define `get_tag_info` closure: joins `tag` and `tag_name` tables, filters by name, returns `(id, last_edit_time)`.
3. Open a database transaction.
4. Look up the source tag. Return `NotFound` if absent.
5. Look up the target tag. Return `NotFound` if absent.
6. If IDs are equal, return `ApiError::SelfMerge`.
7. Verify versions of both tags.
8. Call `update::tag::merge` to merge usages, suggestions, and implications from source to target.
9. Record a `merge_snapshot`.
10. After the first transaction, load `TagInfo` for the merged target tag and return as JSON.

---

### 6. `PUT /tag/{name}` — Update a Tag

**Handler:** `update` at `tag.rs:433`

**Authentication:** Different privileges for each field: `TagEditCategory`, `TagEditDescription`, `TagEditName`, `TagEditImplication`, `TagEditSuggestion`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Tag name |

**Request Body:** `TagUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version (required) |
| `category` | `Option<SmallString>` | New category name |
| `description` | `Option<LargeString>` | New description |
| `names` | `Option<Vec<SmallString>>` | New names |
| `implications` | `Option<Vec<SmallString>>` | New implications |
| `suggestions` | `Option<Vec<SmallString>>` | New suggestions |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `TagInfo`

**Error Responses:**
- `403` — Privileges too low
- `404` — Tag does not exist
- `409` — Version outdated or name clash with existing tag
- `422` — Category invalid, name/implication/suggestion invalid, cyclic dependency

**Handler Logic:**
1. Open a database transaction.
2. Load the existing tag from `tag` joined with `tag_name` filtered by name. Return `NotFound` if absent.
3. Verify version.
4. Clone the old tag and retrieve old snapshot data. Clone the snapshot data.
5. For each provided field:
   - **category:** Verify `TagEditCategory`, look up the category ID from `tag_category`, update `category_id` and snapshot.
   - **description:** Verify `TagEditDescription`, update description on snapshot.
   - **names:** Verify `TagEditName`. If empty, return `ApiError::NoNamesGiven`. Call `update::tag::set_names`.
   - **implications:** Verify `TagEditImplication`. Resolve/create tag IDs with `FetchMode::Acyclic`. Call `update::tag::set_implications`.
   - **suggestions:** Verify `TagEditSuggestion`. Resolve/create tag IDs with `FetchMode::Acyclic`. Call `update::tag::set_suggestions`.
6. Update `last_edit_time` to now.
7. Save the tag via `save_changes`.
8. Record a `modification_snapshot`.
9. After the first transaction, load `TagInfo` and return as JSON.

---

### 7. `DELETE /tag/{name}` — Delete a Tag

**Handler:** `delete` at `tag.rs:528`

**Authentication:** Requires `Action::TagDelete`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Tag name |

**Request Body:** `DeleteBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |

**Successful Response:** `200` with `()`

**Error Responses:**
- `403` — Privileges too low
- `404` — Tag does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Verify `Action::TagDelete` privilege.
2. Open a database transaction.
3. Load the tag from `tag` joined with `tag_name` filtered by name. Return `NotFound` if absent.
4. Verify version.
5. Retrieve snapshot data for the tag.
6. Record a `deletion_snapshot`.
7. Delete the tag from `tag::table`.
8. Return `Json(())`.

---

## Type Definitions

### `TagCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `category` | `SmallString` | Category name (must exist) |
| `description` | `Option<LargeString>` | Tag description |
| `names` | `Vec<SmallString>` | Tag names |
| `implications` | `Option<Vec<SmallString>>` | Implied tags |
| `suggestions` | `Option<Vec<SmallString>>` | Suggested tags |

### `TagUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |
| `category` | `Option<SmallString>` | New category |
| `description` | `Option<LargeString>` | New description |
| `names` | `Option<Vec<SmallString>>` | New names |
| `implications` | `Option<Vec<SmallString>>` | New implications |
| `suggestions` | `Option<Vec<SmallString>>` | New suggestions |

### `TagSibling`

| Field | Type | Description |
|-------|------|-------------|
| `tag` | `TagInfo` | The sibling tag |
| `occurrences` | `i64` | Co-occurrence count |

### `TagSiblings`

| Field | Type | Description |
|-------|------|-------------|
| `results` | `Vec<TagSibling>` | Sibling list (max 50, sorted by occurrences descending) |
