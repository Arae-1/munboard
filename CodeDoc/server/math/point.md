# Math — 2D Integer Point (`math/point.rs`)

## Summary

This module defines `IPoint2<T>`, a point on a 2D integer lattice. It supports construction, lexicographical ordering, and conversion between integer types.

## Imports

- `num_traits::int::PrimInt`: Integer operations trait.
- `std::convert::TryFrom`: Fallible type conversion.
- `std::num::TryFromIntError`: Conversion error type.

## Struct: `IPoint2<T>`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IPoint2<T> {
    pub i: T,
    pub j: T,
}
```

Represents a point at coordinates `(i, j)`. `i` is the first axis (row/x), `j` is the second axis (column/y). Derives `PartialOrd` and `Ord` with lexicographical ordering (`i` first, then `j`).

### Method: `IPoint2::new(i: T, j: T) -> Self`

Constructs a new point. Available where `T: PrimInt`.

## Trait: `TryFrom<IPoint2<i64>> for IPoint2<u32>`

Converts an `i64` point to a `u32` point. Fails with `TryFromIntError` if either coordinate is negative or exceeds `u32::MAX`.

**Conversion order:** Converts `j` first, then `i`.

## Tests: `lexicographical_ordering`

Verifies ordering rules:
- `(1, 0) > (0, 0)` — larger i wins.
- `(0, 1) > (0, 0)` — same i, larger j wins.
- `(1, 0) > (0, 1)` — i has priority over j.
- `(0, 8) < (1, 0)` — i priority always.
- `(1, 1) > (1, 0)` — same i, j comparison.
- `(1, 1) < (1, 2)` — j increases.
- `(3, 0) > (2, 2)` — i priority.
