# Math Utilities

## Individual Files

- [cartesian.md](cartesian.md)
- [interval.md](interval.md)
- [point.md](point.md)
- [rect.md](rect.md)

## Purpose

The math module provides geometric primitives for image annotation and image signature computation. It includes a 2D integer point, integer interval, integer rectangle, Cartesian product, and a 2D array.

## Integer Point (`point.rs`)

### `IPoint2<T>`
A point on a two-dimensional integer lattice with coordinates `i` and `j`. Uses `(i, j)` rather than `(x, y)` convention.

Supports lexicographical ordering (`#[derive(PartialOrd, Ord)]`). Points are compared by `i` first, then `j`. This means `IPoint2(1, 0) > IPoint2(0, 9)`.

### `TryFrom<IPoint2<i64>> for IPoint2<u32>`
Converts signed grid points to unsigned pixel coordinates. Fails if coordinates are negative or exceed u32::MAX.

## Integer Interval (`interval.rs`)

### `Interval<T>`
Represents an integer interval with inclusive min and max bounds.

**Fields**: `start` (inclusive), `end` (exclusive).

### Methods
- `new(min, max)`: Creates an interval from inclusive bounds.
- `min()`: Returns the lowest contained value.
- `max()`: Returns the highest contained value.
- `midpoint()`: Calculates the midpoint, avoiding overflow.
- `length()`: Returns the number of points (end - start). Returns 0 if empty.
- `contains(value)`: Checks if a value is within the interval.
- `is_empty_set()`: Returns true if start >= end.
- `intersection(a, b)`: Computes the intersection of two intervals.
- `linspace<N>()`: Creates an evenly spaced array of N points within the interval.
- `shrink(n)`: Reduces both bounds by `n` units. If this would make the interval empty, collapses to a single point at the midpoint.

### `linspace` behavior
- N=0: Returns all zeros.
- N=1: Returns the midpoint.
- N>=2: Returns evenly spaced points from min to max.

## Integer Rectangle (`rect.rs`)

### `IRect<T>`
A box on a two-dimensional integer lattice, defined by `i_bounds` and `j_bounds` (both Intervals).

### Methods
- `new(i_bounds, j_bounds)`: Constructs from two intervals.
- `new_zero_based(i_max, j_max)`: Creates a rectangle from (0, 0) to (i_max, j_max).
- `new_centered_square(center, radius)`: Creates a square centered at a point extending `radius` in all directions.
- `contains(point)`: Checks if a point is within the rectangle.
- `intersection(a, b)`: Computes the intersection of two rectangles.
- `total_points()`: Returns the total number of integer points. Returns None on overflow.
- `min_corner()`: Returns the lower-left corner point.
- `max_corner()`: Returns the upper-right corner point.
- `iter()`: Returns an iterator over all points within the rectangle, in row-major order.

### `IRectIter<T>`
An iterator that yields `IPoint2<T>` values. Starts at the min corner, increments `j` first, then `i`. Uses lexicographic ordering.

### `Array2D<T, N, M>`
A fixed-size two-dimensional array for storing values at grid positions.

### Methods
- `new(initial_value)`: Fills the entire array with one value.
- `bounds<I>()`: Returns the array's bounds as an IRect.
- `at(index)`: Reads a value at a position. Panics on out-of-bounds.
- `set_at(index, value)`: Writes a value at a position. Panics on out-of-bounds.
- `get(index)`: Reads a value at a position. Returns None on out-of-bounds (checks bounds first).
- `iter()`: Returns a flat iterator over all elements.
- `indexed_iter<I>()`: Returns pairs of (index, value) for all elements.

## Cartesian Product (`cartesian.rs`)

### `CartesianProduct<L, R, N, M>`
Represents the Cartesian product of two fixed-size arrays `left` (size N) and `right` (size M). Used to compute the product of grid point coordinate sets.

### Methods
- `new(left, right)`: Constructs from two arrays.
- `at(index)`: Returns the pair at a given position.
- `dimensions()`: Returns the product bounds as an IRect.
- `iter()`: Returns an iterator yielding `(&L, &R)` pairs.
- `indexed_iter()`: Returns an iterator yielding `(IPoint2<usize>, (&L, &R))` pairs.

### `CartesianProductIter`
Iterates in row-major order (right index changes fastest). Implements `size_hint` returning the product of the two lengths.
