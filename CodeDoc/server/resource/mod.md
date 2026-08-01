# Module Root — `resource/mod.rs`

**Source:** `server/src/resource/mod.rs` (132 lines)

## Purpose

This file is the root of the resource module. It declares all submodules. It also supplies shared helper functions that every resource type uses. These helpers reorder batches, map optional relations, and collect name maps.

## Submodules

| Submodule | File | Purpose |
|-----------|------|---------|
| `comment` | `comment.rs` | Comment resource serialization |
| `custom_field` | `custom_field.rs` | Custom field resource serialization |
| `field` | `field.rs` | Mask and Batcher utility types |
| `pool` | `pool.rs` | Pool resource serialization |
| `pool_category` | `pool_category.rs` | Pool category resource serialization |
| `post` | `post.rs` | Post resource serialization |
| `snapshot` | `snapshot.rs` | Snapshot resource serialization |
| `tag` | `tag.rs` | Tag resource serialization |
| `tag_category` | `tag_category.rs` | Tag category resource serialization |
| `user` | `user.rs` | User resource serialization |
| `user_token` | `user_token.rs` | User token resource serialization |

## Dependencies

- `crate::string::SmallString` — compact string type for tag names and identifiers
- `diesel::Identifiable` — trait that exposes primary keys
- `std::collections::HashMap` — for name maps
- `std::sync::Arc` — for shared ownership of name slices

## Functions

### `single<T>(batch: Vec<T>) -> T`

Extract exactly one element from a vector. Use this after `new_batch` when you know the input is a single entity.

**Panics:** If the vector does not contain exactly one element.

**Usage pattern:**
```rust
Self::new_batch(conn, ctx, vec![entity], fields).map(resource::single)
```

### `order_as<T>(values: Vec<T>, order: &[i64]) -> Vec<T>`

Reorder a vector of identifiable items. The result has items in the same sequence as the `order` slice of primary keys. The items must have the `Identifiable` trait where `Id = &i64`.

**Algorithm:** O(n²) scan-and-swap. Acceptable for batches of 40-50 items. Starts at index 0. Finds the item whose primary key matches `order[index]`. If the item is at a different position, swaps it. Panics if `values.len() != order.len()` or if IDs are not unique.

**Use cases:**
- `new_batch_from_ids` reorders database results to match input ID order
- Database queries do not guarantee the same order as the input slice

### `order_transformed_as<V, F>(values: Vec<V>, order: &[i64], get_id: F) -> Vec<V>`

Same algorithm as `order_as`. Uses a custom function `get_id` to extract primary keys instead of the `Identifiable` trait. Use this when you load tuples from the database where the first element is the ID.

**Example:** A query returns `Vec<(i64, SmallString)>` (id, name). Use `order_transformed_as` with `|&(id, _)| id` to reorder by the tuple's first element.

### `order_like<V, T, F>(unordered_values: Vec<V>, ordered_values: &[T], get_id: F) -> Vec<Option<V>>`

Map unordered values into an Option vector. The result has the same length as `ordered_values`. Each position contains `Some(value)` if a matching value exists, or `None` if it does not.

**Use pattern:** Load related data that may not exist for every parent entity. For example, not every comment has an owner (anonymous comments). The result places the owner at the matching position or `None`.

**Constraints:**
- `unordered_values.len() <= ordered_values.len()`
- Every value's ID must appear in `ordered_values`
- Panics otherwise

### `collect_names(ordered_names: Vec<(i64, SmallString)>) -> HashMap<i64, Arc<[SmallString]>>`

Build a map from a resource ID to its list of names. The input must be sorted by (resource_id, name_order). The function detects boundaries where consecutive entries belong to different resource IDs.

**Algorithm:**
1. Scan `ordered_names` to find boundaries where the resource ID changes
2. For each boundary range, collect names into a `Vec` with exact capacity
3. Convert the Vec into `Arc<[SmallString]>` for shared ownership
4. Insert into the HashMap

**Memory:** Pre-allocates exact capacity. Avoids reallocation when converting to `Arc`. Each resource type (tags, pools) can share the same `Arc` name slices across multiple `MicroTag` or `MicroPool` references.

**Used by:** `post.rs` (post tags, pool names), `tag.rs` (implications, suggestions).

## Design Note

The module relies on `diesel::GroupedBy` preserving relative order between parent and child elements. The implementation currently supports this behavior. If the behavior changes, the `order_like` and `order_transformed_as` functions provide equivalent ordering guarantees.
