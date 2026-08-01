# Field Selection System — `resource/field.rs`

**Source:** `server/src/resource/field.rs` (100 lines)

## Purpose

This file supplies the field-level selection system for API responses. Clients can request specific fields via a query parameter. The `Mask` type encodes which fields to include. The `Batcher` type executes database queries only for requested fields.

## Dependencies

- `serde::{Deserialize, Deserializer}` — custom deserialization from query strings
- `std::marker::PhantomData` — type-level tagging for Mask
- `std::ops::{BitOr, BitOrAssign, Index}` — operator implementations
- `std::str::FromStr` — parsing field names from strings

## Types

### `Mask<F>`

A 64-bit bitmask. Each bit corresponds to one field in the resource's `Field` enum. The generic parameter `F` ties the mask to a specific resource type.

**Fields:**
- `value: u64` — the raw bitmask value
- `phantom: PhantomData<F>` — zero-sized type marker

**Constructor methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `new()` | `-> Self` | Create empty mask (no fields selected) |
| `from_u64(value)` | `u64 -> Self` | Create mask from raw bits |

**Access method:**
- `contains(field: F) -> bool` — check if a field is enabled. Tests `value & (1 << field.into()) != 0`.

**Trait implementations:**

| Trait | Behavior |
|-------|----------|
| `Index<F>` | Returns `&true` if field is set, `&false` otherwise. Enables `mask[Field::Id]` syntax. |
| `BitOr<F>` | Returns a new Mask with the field added. Enables `mask \| Field::Id` syntax. |
| `BitOrAssign<F>` | Adds the field to the mask in-place. |
| `From<&[F]>` | Build mask from a slice of fields. Folds with `\|` over the slice. |
| `From<[F; N]>` | Build mask from a fixed-size array. Delegates to `From<&[F]>`. |

**Deserialization:**

```rust
impl Deserialize for Mask<F>
```

Parses a comma-separated field list from a query string. Each field name is parsed via `F::from_str`. The `EnumString` derive macro on each `Field` enum enables this parsing with `#[strum(serialize_all = "camelCase")]`.

- If the query parameter is present (e.g. `?fields=id,name,usages`): builds mask with those fields only
- If the query parameter is absent or null (`None`): returns a mask with all bits set (`u64::MAX`), meaning **all fields**

### `Batcher<F>`

Lazy batch loader for related data. Stores the field mask and the batch size. Each `exec` call checks whether a field is requested before running the expensive database query.

**Fields:**
- `enabled_fields: Mask<F>` — which fields to load
- `batch_size: usize` — expected number of results

**Constructor:**
- `new(enabled_fields: Mask<F>, batch_size: usize) -> Self`

**Method:**

```rust
fn exec<T, E, G>(&self, field: F, function: G) -> Result<Vec<T>, E>
where
    G: FnMut() -> Result<Vec<T>, E>,
```

Conditionally executes a data-loading closure.

**Behavior:**
1. If `field` is set in the mask: calls `function()`. Asserts the result length equals `batch_size`. Returns the result.
2. If `field` is not set: returns an empty `Vec`. The closure is never called.

**Why this pattern exists:** Resource batch construction iterates over model entities in reverse, popping pre-loaded vectors. When a field is not requested, the `Batcher` returns an empty vector. Each `.pop()` on the empty vector returns `None`, which produces `None` in the serialized output (skipped by `skip_serializing_none`). This avoids unnecessary database queries for unrequested fields.

## Usage Pattern

Every resource file follows this pattern:

```rust
let f = Batcher::new(fields, entities.len());
let mut categories = f.exec(Field::Category, || get_categories(conn, &entities))?;
let mut names = f.exec(Field::Names, || get_names(conn, &entities))?;

let mut results = entities
    .into_iter()
    .rev()
    .map(|entity| Self {
        category: categories.pop(),
        names: names.pop(),
        // ...
    })
    .collect::<Vec<_>>();
results.reverse();
```

The `.rev()` / `results.reverse()` pattern exists because `.pop()` removes from the end. By reversing first, the first entity in the batch gets the first pre-loaded value.
