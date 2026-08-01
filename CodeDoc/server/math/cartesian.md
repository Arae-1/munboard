# Math — Cartesian Product (`math/cartesian.rs`)

## Summary

This module provides `CartesianProduct`, a generic type representing the Cartesian product of two fixed-size arrays. It supports indexed iteration over all pairs and random access by 2D index. Space complexity is O(N + M).

## Imports

- `math::point::IPoint2`: 2D point type.
- `math::rect::IRect`: 2D rectangle used for dimensions.

## Struct: `CartesianProduct<L, R, const N: usize, const M: usize>`

```rust
pub struct CartesianProduct<L, R, const N: usize, const M: usize> {
    left: [L; N],
    right: [R; M],
}
```

Represents the Cartesian product of two fixed-size arrays. `left × right` has `N * M` total pairs.

**Fields:**
- `left: [L; N]`: The first set (N elements).
- `right: [R; M]`: The second set (M elements).

### Method: `CartesianProduct::new(left, right) -> Self`

Constructs a new Cartesian product from two arrays. Takes ownership of both.

### Method: `CartesianProduct::at(index: IPoint2<usize>) -> (&L, &R)`

Returns references to the elements at the given 2D index. `index.i` selects from `left`, `index.j` selects from `right`. Panics on out-of-bounds access.

### Method: `CartesianProduct::left_set() -> &[L]` (test only)

Returns the left set as a slice. Only available in test builds.

### Method: `CartesianProduct::right_set() -> &[R]` (test only)

Returns the right set as a slice. Only available in test builds.

### Method: `CartesianProduct::dimensions() -> IRect<usize>`

Returns the dimensions of the Cartesian product as a zero-based `IRect` from `(0, 0)` to `(N-1, M-1)`.

### Method: `CartesianProduct::iter() -> CartesianProductIter`

Returns a row-major iterator over all pairs. Iterates by incrementing `j` from 0 to M-1, then incrementing `i` and resetting `j`. See `CartesianProductIter`.

### Method: `CartesianProduct::indexed_iter() -> impl Iterator<Item = (IPoint2<usize>, (&L, &R))>`

Returns an iterator that yields both the 2D index and the pair. Implemented as `self.dimensions().iter().zip(self.iter())`.

## Struct: `CartesianProductIter<'a, L, R, const N: usize, const M: usize>`

```rust
pub struct CartesianProductIter<'a, L, R, const N: usize, const M: usize> {
    cartesian_product: &'a CartesianProduct<L, R, N, M>,
    current: IPoint2<usize>,
}
```

Iterator over the pairs of a Cartesian product. Starts at `(0, 0)` and proceeds row-major.

### Trait: `Iterator for CartesianProductIter`

**Item:** `(&'a L, &'a R)`

**`next()` method:**
1. If `self.current > self.cartesian_product.dimensions().max_corner()`: return `None` (past the end).
2. Save the current position.
3. Advance:
   - If `current.j < right.len() - 1`: increment `j`.
   - Otherwise: reset `j = 0` and increment `i`.
4. Return `Some(self.cartesian_product.at(current))`.

**`size_hint()` method:**
Returns `(N * M, Some(N * M))`. Computes with `checked_mul`, returning `usize::MAX` on overflow (unlikely for reasonable N, M).

## Tests: `cartesian_product_iteration`

Verifies sum of all products of two arithmetic progressions matches the formula `n*(n+1)*m*(m+1)/4` for various sizes of Cartesian products.
