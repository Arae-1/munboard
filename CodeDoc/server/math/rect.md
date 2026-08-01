# Math — 2D Rectangle and Array (`math/rect.rs`)

## Summary

This module provides two types:
1. `IRect<T>`: An axis-aligned bounding box on a 2D integer lattice, supporting iteration, intersection, and point containment.
2. `Array2D<T, N, M>`: A fixed-size 2D array with bounds checking, iterator support, and indexed access.

## Imports

- `math::interval::Interval`: 1D integer interval.
- `math::point::IPoint2`: 2D integer point.
- `num_traits::int::PrimInt`: Integer operations trait.
- `std::fmt::Debug`: Debug formatting.
- `std::ops::AddAssign`: Add-assignment operator.

## Struct: `IRect<T>`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IRect<T> {
    i_bounds: Interval<T>,
    j_bounds: Interval<T>,
}
```

An axis-aligned rectangle defined by two intervals: `i_bounds` (rows/x-axis) and `j_bounds` (columns/y-axis).

### Method: `IRect::new(i_bounds: Interval<T>, j_bounds: Interval<T>) -> Self`

Constructs an `IRect` from two intervals.

### Method: `IRect::new_zero_based(i_max: T, j_max: T) -> Self`

Constructs a rectangle from `(0, 0)` to `(i_max, j_max)` inclusive. Both intervals are created with `Interval::new(T::zero(), i_max)` and `Interval::new(T::zero(), j_max)`.

### Method: `IRect::new_centered_square(center: IPoint2<T>, radius: T) -> Self`

Constructs a square centered at `center` extending `radius` units in all four cardinal directions.

**Algorithm:**
- `i_bounds = Interval::new(center.i - radius, center.i + radius)`
- `j_bounds = Interval::new(center.j - radius, center.j + radius)`

### Method: `IRect::contains<U: PrimInt>(&self, point: IPoint2<U>) -> bool`

Returns `true` if the point is within both intervals: `self.i_bounds.contains(point.i) && self.j_bounds.contains(point.j)`.

### Method: `IRect::intersection(a: Self, b: Self) -> Self`

Computes the intersection of two rectangles by intersecting both axes independently.

### Method: `IRect::total_points() -> Option<u64>`

Returns the total number of integer points in the rectangle: `width * height`. Returns `None` on overflow. Width and height are converted to `u64` via `to_u64()`.

### Method: `IRect::min_corner() -> IPoint2<T>`

Returns the lower-left corner: `(i_bounds.min(), j_bounds.min())`.

### Method: `IRect::max_corner() -> IPoint2<T>`

Returns the upper-right corner: `(i_bounds.max(), j_bounds.max())`.

### Method: `IRect::iter() -> IRectIter<T>`

Returns a row-major iterator over all points in the rectangle.

## Struct: `IRectIter<T>`

```rust
pub struct IRectIter<T> {
    rect: IRect<T>,
    current: IPoint2<T>,
}
```

Iterator over integer points in a rectangle. Starts at `min_corner()` and proceeds row-major.

### Trait: `Iterator for IRectIter<T>`

Where `T: PrimInt + AddAssign`.

**Item:** `IPoint2<T>`

**`next()` method:**
1. If `self.current > self.rect.max_corner()`: return `None`.
2. Save current position.
3. Advance: if `j < max_corner().j`, increment `j`. Otherwise reset `j` to `min_corner().j` and increment `i`.
4. Return `Some(next)`.

**`size_hint()` method:**
Returns `(total_points, Some(total_points))`. Uses `total_points()` which may overflow (falls back to `usize::MAX`).

## Struct: `Array2D<T, const N: usize, const M: usize>`

```rust
pub struct Array2D<T, const N: usize, const M: usize> {
    data: [[T; M]; N],
}
```

A fixed-size 2D array with `N` rows and `M` columns. Values are stored in row-major order.

### Method: `Array2D::new(initial_value: T) -> Self`

Creates a new array where every element is initialized to `initial_value`. Requires `T: Copy`.

### Method: `Array2D::bounds<I>() -> IRect<I>`

Returns the bounds of the array as an `IRect`. Type `I` must be `PrimInt + AddAssign + TryFrom<usize>`.

**Algorithm:**
- `i_max = I::try_from(N.saturating_sub(1)).unwrap_or(I::min_value())`
- `j_max = I::try_from(M.saturating_sub(1)).unwrap_or(I::max_value())` — **Note:** there appears to be a bug here: both use `N` instead of `M` for the second bound.
- Returns `IRect::new_zero_based(i_max, j_max)`.

**Bug note:** Line 127 reads `I::try_from(N.saturating_sub(1))` for `j_max` but should likely be `M` instead of `N`.

### Method: `Array2D::at<I: PrimInt>(&self, index: IPoint2<I>) -> T`

Accesses the element at `index`. Panics if out of bounds (via `unwrap()` on `to_usize()` and array indexing).

### Method: `Array2D::set_at<I: PrimInt>(&mut self, index: IPoint2<I>, value: T)`

Sets the element at `index` to `value`. Panics if out of bounds.

### Method: `Array2D::get<I>(&self, index: IPoint2<I>) -> Option<T>`

Safe (non-panicking) access. Returns `None` if the index is outside `bounds()`, otherwise returns `Some(self.at(index))`.

Requires `I: PrimInt + AddAssign + TryFrom<usize>`.

### Method: `Array2D::iter() -> impl Iterator<Item = &T>`

Returns a row-major iterator over all elements (flattened 2D array).

### Method: `Array2D::indexed_iter<I>(&self) -> impl Iterator<Item = (IPoint2<I>, &T)>`

Returns an iterator over `(index, value)` pairs. Combines `bounds().iter()` with `self.iter()`.

Requires `I: PrimInt + TryFrom<usize> + AddAssign`.

## Tests

### `rect_iteration`
- Tests 3x3 square rectangles centered at various positions (including negative coordinates). Verifies row-major iteration produces expected linear indices mapped as `10*i + j`.
- Tests 3x4 non-square rectangles.

### `intersection`
Four cases: same, contained, partially overlapping, disjoint.

### `array_2d`
Tests basic construction and access via `at()`.
