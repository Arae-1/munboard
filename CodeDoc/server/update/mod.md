# Update Module — Core Framework

**File:** `server/src/update/mod.rs` (8 lines)
**Type:** module root
**Purpose:** Declares submodules and provides a critical security note.

---

## Submodules Declared

| Submodule | Purpose |
|---|---|
| `pool` | Pool post count recalculation, name management, pool merging |
| `post` | Post tag assignment, relation management, note updates, post merging, thumbnail updates |
| `tag` | Tag implication chain resolution, tag merging, name management, suggestion/implication management |
| `user` | User login time tracking, edit time tracking, avatar updates |

---

## Important Note (from source)

> **NOTE:** Unless documented otherwise, the functions in this module do not check that the client has the required privileges to perform their respective actions. Make sure to check for privileges before calling them, if necessary.

All functions in the `update` module assume the caller has already performed privilege checks. They operate on database records directly. Callers must call `ctx.verify_privilege(...)` or equivalent before invoking these functions.

**Exception:** `tag::get_or_create_tag_ids` explicitly checks `Action::TagCreate` privileges when creating new tags.

---

## Pattern

The update module functions are **side-effect heavy**. They:
- Delete and re-insert rows (replacement pattern for ordered collections)
- Mutate multiple tables in a single transaction
- Cascade updates to related resources (e.g., timestamps)
- Perform filesystem operations (thumbnails, avatars, post files)

All functions require `&mut PgConnection` and operate within a database transaction context. Most return `ApiResult<()>` or `QueryResult<()>`.
