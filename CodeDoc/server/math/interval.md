# Math — Integer Interval (`math/interval.rs`)

## Summary

This module provides `Interval`, a generic integer interval type with inclusive min, exclusive max representation. It supports length, containment, intersection, evenly-spaced point arrays (linspace), and shrinking.

## Imports

- `num_traits::PrimInt`: Trait for primitive integer operations.

## Struct: `Interval<T>`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interval<T> {
    start: T,
    end: T,
}
```

Represents a half-open interval `[start, end)`. `start` is inclusive, `end` is exclusive. Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

### Method: `Interval::new(min: T, max: T) -> Self`

Creates an inclusive interval from `min` to `max`.

**Algorithm:**
- Set `start = min`.
- Set `end = max + 1` (convert inclusive max to exclusive end).

Example: `Interval::new(0, 5)` represents `[0, 6)` with `start=0, end=6`, containing values 0, 1, 2, 3, 4, 5.

### Method: `Interval::min() -> T`

Returns the lowest point in the interval (same as `start`).

### Method: `Interval::max() -> T`

Returns the highest point in the interval: `end - 1`.

### Method: `Interval::midpoint() -> T`

Calculates the midpoint, avoiding overflow: `start + (end - start) / 2`.

### Method: `Interval::length() -> T`

Returns the number of points in the interval. Returns `0` if the interval is empty (`start >= end`). Otherwise returns `end - start`.

### Method: `Interval::contains<U: PrimInt>(value: U) -> bool`

Returns `true` if `value` is in `[start, end)`. The value is first converted to `T` via `T::from()`. If conversion fails, returns `false`.

### Method: `Interval::is_empty_set() -> bool`

Returns `true` if `start >= end` (no points in the interval).

### Method: `Interval::intersection(a: Self, b: Self) -> Self`

Computes the intersection of two intervals.

**Algorithm:**
- `start = max(a.start, b.start)`
- `end = min(a.end, b.end)`
- Return `Interval { start, end }`.

If the intervals are disjoint, the result will be an empty set (`start >= end`).

### Method: `Interval::linspace<const N: usize>() -> [T; N]`

Computes `N` evenly-spaced points within the interval.

**Algorithm by N:**
- **N = 0:** Return `[T::zero(); 0]` (empty array).
- **N = 1:** Return `[self.midpoint(); 1]`.
- **N >= 2:**
  1. Convert `min` and `max` to `f64`.
  2. `step = (max_f64 - min_f64) / (N - 1)`.
  3. For `index` in `0..N`: `point = (min_f64 + index * step).round()`.
  4. Convert `point` back to `T` and store.

Uses `f64` for the computation to handle floating-point precision for non-integer divisions. The final values round to the nearest integer.

### Method: `Interval::shrink(&mut self, n: T)`

Shrinks the interval by `n` on both sides. Uses saturating arithmetic.

**Algorithm:**
1. `start = start.saturating_add(n)` (increase start by `n`, clamp at max).
2. `end = end.saturating_sub(n)` (decrease end by `n`, clamp at min).
3. If the interval becomes empty (`is_empty_set()`):
   - Collapse to a single-point interval at `midpoint`.
   - `start = midpoint`, `end = midpoint + 1`.

This ensures the interval always has at least one point after shrinking.

## Tests

### `intersection`
Four test cases:
- Same interval: intersection equals the interval.
- One contained in another: returns the smaller.
- Partially overlapping: returns the overlap range.
- Disjoint: returns an empty set.

### `create_linspace`
Tests `linspace` with various N values for `Interval::new(0, 2)` and `Interval::new(0, 5)` and `Interval::new(0, 100)` and `Interval::new(100, 0)` (descending interval). Verifies exact expected arrays.
