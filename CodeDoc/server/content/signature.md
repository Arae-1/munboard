# Image Signature (`content/signature.rs`)

## Summary

This module implements a perceptual image hashing algorithm for reverse image search. It follows the algorithm from H. Chi Wong, Marshall Bern, and David Goldberg with modifications. The signature allows measuring visual similarity between images. Each image produces a fixed-length compressed signature and a set of searchable word indexes.

## Imports

- `math::cartesian::CartesianProduct`: Grid point generation.
- `math::interval::Interval`: Integer interval operations.
- `math::point::IPoint2`: 2D point type.
- `math::rect::{Array2D, IRect}`: 2D array and rectangle.
- `image::{DynamicImage, GrayImage}`: Image types.
- `num_traits::ToPrimitive`: Numeric conversions.
- `std::num::NonZeroU64`: Non-zero integer for safe division.

## Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `CROP_PERCENTILE` | `1` | Percentile threshold for border cropping |
| `GRID_SIZE` | `9` | Size of the square grid used for signature (9x9 = 81 points) |
| `IDENTICAL_TOLERANCE` | `1` | Pixel intensity differences <= 1 are treated as identical |
| `LUMINANCE_LEVELS` | `2` | Number of brightness levels (1 dark, 1 light, + 1 neutral = 3 symbols) |
| `NUM_LETTERS` | `12` | Length of each search index word |
| `NUM_SYMBOLS` | `2 * LUMINANCE_LEVELS + 1 = 5` | Number of possible values a "letter" can take |
| `SIGNATURE_LEN` | `8*(9-2)^2 + 20*(9-2) + 12 = 544` | Total signature length in bytes |
| `SIGNATURE_DIGITS` | `ilog(5, u64::MAX) - 1 = 26` | Number of base-5 digits per i64 word (fits in 64 bits) |
| `NUM_WORDS` | `100` | Number of search indexes to generate |
| `COMPRESSED_SIGNATURE_LEN` | `544 div_ceil(26) = 21` | Compressed signature length in i64 values |
| `SIGNATURE_VERSION` | `1` | Version number, bumped when algorithm changes |

**Derivation of `SIGNATURE_LEN`:**
The 9x9 grid produces intensity differences for each grid point with its neighbors. Each interior grid point (7x7 = 49) has 8 neighbors. Edge points and corner points are excluded. So: 49 points × 8 neighbors = 392. Plus corrections: `8*(9-2)^2 + 20*(9-2) + 12 = 8*49 + 20*7 + 12 = 392 + 140 + 12 = 544`.

## Type Alias: `GridPoints`

```rust
type GridPoints = CartesianProduct<u32, u32, GRID_SIZE, GRID_SIZE>;
```

The Cartesian product of two sets of 9 u32 coordinates. Produces 81 grid points (9x9).

## Struct: `SignatureCache`

```rust
pub struct SignatureCache {
    signature: [u8; SIGNATURE_LEN],
    norm: f64,
}
```

Stores an uncompressed signature and its pre-computed L2 norm. The norm is used in distance calculation to avoid recomputing it for the query signature.

### Method: `SignatureCache::new(compressed_signature) -> Self`

Creates a cache from a compressed signature.

**Algorithm:**
1. Call `uncompress(compressed_signature)` to get `[u8; 544]`.
2. Call `norm(&signature)` to compute the L2 norm.
3. Return `SignatureCache { signature, norm }`.

## Function: `compute(image: &DynamicImage) -> [i64; COMPRESSED_SIGNATURE_LEN]`

```rust
pub fn compute(image: &DynamicImage) -> [i64; COMPRESSED_SIGNATURE_LEN]
```

Computes a compressed image signature. This is the main entry point.

**Algorithm (step by step):**

### Step 1: Convert to grayscale
```rust
let gray_image = image.to_luma8();
```
The input image is converted to an 8-bit grayscale (`Luma8`) image.

### Step 2: Compute grid points
```rust
let (grid_points, grid_square_radius) = compute_grid_points(&gray_image);
```
Determines where the 9x9 grid of analysis points should be placed on the image and the size of each grid square.

### Step 3: Compute intensity matrix
```rust
let mean_matrix = compute_intensity_matrix(&gray_image, &grid_points, grid_square_radius);
```
For each of the 81 grid points, computes the average pixel intensity within a square region of radius `grid_square_radius`.

### Step 4: Compute differences
```rust
let differences = compute_differences(&mean_matrix);
```
For each interior grid point and each of its 8 neighbors, computes `neighbor_value - center_value`. This produces 392 signed differences (49 interior points × 8 neighbors). Boundary points are excluded.

### Step 5: Normalize
```rust
let signature = normalize(&differences);
```
Maps each difference value to a symbol in `[0, NUM_SYMBOLS)` (0 to 4) based on its sign and magnitude relative to other differences.

### Step 6: Compress
```rust
compress(&signature)
```
Packs the 544 base-5 digits into 21 `i64` values for efficient storage. Each `i64` stores 26 base-5 digits.

## Function: `distance(signature, compressed_signature) -> f64`

```rust
pub fn distance(signature: &SignatureCache, compressed_signature: &[i64; COMPRESSED_SIGNATURE_LEN]) -> f64
```

Computes a similarity distance between two image signatures. The result is in `[0, 1]`. Lower values mean more similar images.

**Algorithm:**
1. Uncompress `compressed_signature` via `uncompress`.
2. Compute L2 squared distance: for each pair of bytes `(a, b)` from the two signatures, compute `(a - b)^2` and sum over all 544 bytes.
3. Take square root: `l2_squared_distance.sqrt()`.
4. Divide by the sum of norms: `l2_distance / (norm_a + norm_b)`.
5. If denominator is `0.0` (identical zero signatures), return `0.0`.

## Function: `generate_indexes(compressed_signature) -> [i32; NUM_WORDS]`

```rust
pub fn generate_indexes(compressed_signature: &[i64; COMPRESSED_SIGNATURE_LEN]) -> [i32; NUM_WORDS]
```

Creates 100 searchable word indexes from a compressed signature. These are used for fast approximate search via PostgreSQL array overlap (`&&` operator).

**Constants:**
- `NUM_REDUCED_SYMBOLS = 3`: Each letter is reduced to 3 possible values (-1, 0, 1).
- `NUM_WORD_DIGITS`: Number of trits (base-3 digits) needed to store `NUM_WORDS` (100). Computed as `ilog(3, 100) + 1 = 5`.
- `CLAMP_VALUE = 1`: Each letter is clamped to `[-1, 1]`.

**Algorithm:**
1. Uncompress the signature.
2. Compute word positions: `Interval::new(0, 544 - 12).linspace::<100>()`. This evenly spaces 100 starting positions across the signature.
3. Extract 100 words of 12 letters each: `signature[pos..pos+12]` for each position.
4. For each word (word_index 0..99):
   - Map each letter (0..4) to a signed difference: `letter - LUMINANCE_LEVELS` (= `letter - 2`), giving values -2, -1, 0, 1, 2.
   - Clamp to `[-1, 1]`.
   - Encode as base-3: sum of `(letter + 1) * 3^position` for each letter. This produces a value in `[0, 3^12 - 1]`.
   - Prefix with word index: `word_index + 3^5 * encoded_letters`.
   - Convert to `i32`.

Each index encodes both the word position and its letter values in a single `i32`, enabling `&&` overlap search.

## Function: `array_from_iter<I, T, const N: usize>(iter: I) -> [T; N]`

Helper that collects exactly `N` items from an iterator into a fixed-size array. Panics in debug if the iterator has fewer than `N` items. Uses `Default` for initial values and iterates to fill. Returns the array.

## Function: `crop_position(delta_iter, total_delta) -> usize`

Finds the crop position on one edge of an image by summing column/row deltas until a percentile threshold is reached.

**Algorithm:**
1. `delta_limit = CROP_PERCENTILE * total_delta / 100` (1% of total).
2. Chain a leading `0` to the delta iterator.
3. Use `scan` to compute cumulative delta.
4. Return the first position where cumulative delta >= delta_limit, or 0.

## Function: `crop(deltas: &[u64]) -> Interval<u32>`

Determines crop bounds for one dimension.

**Algorithm:**
1. `total_delta = deltas.iter().sum()`.
2. Lower bound: `crop_position(deltas.iter().copied(), total_delta)` (scan from start).
3. Upper bound: `deltas.len().saturating_sub(1) - crop_position(deltas.iter().rev().copied(), total_delta)` (scan from end).
4. Convert to `u32`, return `Interval::new(lower, upper)`.

This removes uniform borders (often solid black/white bars) from the analysis area.

## Function: `compute_row_deltas(image: &GrayImage) -> Vec<u64>`

Computes the sum of absolute differences between each row and the next row.

**Algorithm:**
1. Get flat pixel samples.
2. For each row index `j` (0 to height-2):
   - Get row `j` and row `j+1` from flat samples.
   - For each pixel pair: `u64::from(pixel.abs_diff(next_pixel))`.
   - Sum for the entire row.
3. Return a `Vec<u64>` of length `height - 1`.

## Function: `compute_column_deltas(image: &GrayImage) -> Vec<u64>`

Computes the sum of absolute differences between each column and the next column. Iterates over rows for better cache locality.

**Algorithm:**
1. Initialize `deltas = vec![0; image_width - 1]`.
2. For each row:
   - Get row slice from flat samples.
   - For each column delta `(delta, [pixel, next_pixel])`: add `u64::from(pixel.abs_diff(next_pixel))` to `delta`.
3. Return `deltas`.

## Function: `compute_grid_points(image: &GrayImage) -> (GridPoints, u32)`

Determines where the 9x9 grid points are placed and the size of each grid square.

**Algorithm:**
1. Compute row and column deltas.
2. Crop image bounds using `crop()` on both axes.
3. Compute grid square size: `0.5 + min(x_bounds.length(), y_bounds.length()) / 20.0`.
4. Grid square radius = `floor(grid_square_size / 2)`.
5. Shrink x and y bounds by the radius to prevent grid squares from overlapping image borders.
6. Create 9 evenly-spaced x coordinates and 9 y coordinates via `linspace()`.
7. Return `CartesianProduct::new(x_coords, y_coords)` and the radius.

## Function: `compute_intensity_matrix(image, grid_points, grid_square_radius) -> Array2D<u8, 9, 9>`

Computes the average pixel intensity within each grid square.

**Algorithm:**
1. Get image bounds as `IRect`.
2. For each of the 81 grid points:
   - Create a square `IRect` centered on the grid point with given radius.
   - Intersect with image bounds.
   - For each pixel in the intersection: convert to `u64`, read luma value.
   - Sum all luma values.
   - Average = `sum / grid_square.total_points()`.
   - Store as `u8` in the 9x9 array at the corresponding position.
3. Return the `Array2D`.

## Function: `compute_differences(intensity_matrix) -> [i16; SIGNATURE_LEN]`

Computes intensity differences between each grid point and its 8 neighbors.

**Algorithm:**
1. For each of the 81 positions `(i, j)` in the 9x9 matrix:
   - For each of 8 neighbors (3x3 square centered on point, excluding center):
     - If neighbor is within bounds: `neighbor_value - center_value`.
   - Flat-map all differences into one iterator.
2. Collect exactly `SIGNATURE_LEN` (544) elements via `array_from_iter`.

Only interior points have all 8 neighbors. Boundary points have fewer, and those differences are not included.

## Function: `compute_cutoffs<F>(differences, filter) -> [Option<i16>; LUMINANCE_LEVELS]`

Computes cutoff thresholds that partition the difference values into roughly equal-sized luminance levels.

**Algorithm:**
1. Filter `differences` using predicate `F` (typically `diff < -IDENTICAL_TOLERANCE` for dark cutoffs, `diff > IDENTICAL_TOLERANCE` for light cutoffs).
2. Sort filtered values ascending.
3. `chunk_size = ceil(filtered_values.len() / LUMINANCE_LEVELS)`, minimum 1.
4. Pad with `None` if fewer values than levels: `pad_amount = LUMINANCE_LEVELS - filtered_values.len()`.
5. Split into chunks, take last element of each chunk (the cutoff value). Return `[None; pad_amount]` followed by the chunk last elements.

## Function: `normalize(differences) -> [u8; SIGNATURE_LEN]`

The final normalization step. Maps each difference value to a symbol in `[0, NUM_SYMBOLS)`.

**Algorithm:**
1. Compute dark cutoffs: `compute_cutoffs(differences, |diff| diff < -IDENTICAL_TOLERANCE)`.
2. Compute light cutoffs: `compute_cutoffs(differences, |diff| diff > IDENTICAL_TOLERANCE)`.
3. Build combined cutoffs: `[dark_cutoffs..., Some(IDENTICAL_TOLERANCE), light_cutoffs...]`. This is an array of length `NUM_SYMBOLS` (5): [dark_c0, dark_c1, tolerance, light_c0, light_c1].
4. For each difference: find the first cutoff position where `diff <= cutoff`. Return that position (0-4) as the symbol.
5. Result is `[u8; 544]` where each value is 0-4.

## Function: `compress(uncompressed_signature) -> [i64; COMPRESSED_SIGNATURE_LEN]`

Compresses 544 base-5 digits into 21 `i64` values.

**Algorithm:**
1. Split signature into chunks of `SIGNATURE_DIGITS` (26) digits.
2. Each chunk: encode as `sum(digit * 5^position)` for position 0..25.
3. This produces a `u64` that fits in 26 base-5 digits (5^26 < 2^64).
4. Cast signed: `u64::cast_signed()` converts to `i64`.
5. Collect 21 values.

## Function: `uncompress(compressed_signature) -> [u8; SIGNATURE_LEN]`

Uncompresses 21 `i64` values back to 544 `u8` digits.

**Algorithm:**
1. `DIVISOR = NonZeroU64::new(5)`. Avoids divide-by-zero checks.
2. For each compressed value (cast to `u64`):
   - For each of 26 digit positions:
     - `letter = sum % DIVISOR` (extract least significant base-5 digit).
     - `sum /= DIVISOR`.
     - Convert `letter` to `u8`.
3. Flatten into a `[u8; 544]` array.
4. Return.

## Function: `norm(signature: &[u8; SIGNATURE_LEN]) -> f64`

Computes the L2 norm (Euclidean norm) of an uncompressed signature, centered at zero.

**Algorithm:**
1. For each byte: `(value - LUMINANCE_LEVELS) ^ 2` (center values around 0, squaring gives magnitude).
2. Sum all squared values.
3. Return square root of the sum.

## Tests

### `edge_cases`
Tests 1-pixel, monochrome, and gradient images. Verifies that identical images (monochrome vs 1-pixel) have distance 0 and different ones have non-zero distance.

### `image_signature_regression`
Tests that identical images in different formats (PNG, BMP) produce distance 0. Tests known distance values for similar and different image pairs with tolerance 1e-8. Uses 8-bit grayscale lossless images to avoid decoder variance.

### `signature_robustness`
Tests Lisa Simpson images with varying borders (0, small, large borders, wide crop, cat overlay). Assertions:
- Small border: distance < 0.2
- Large border: distance < 0.2
- Wide aspect: distance < 0.25
- Cat overlay: distance < 0.45
- Starry night (different image): distance > 0.6
- All matching index counts > 0

### `grid_points`
Verifies grid points fall within the actual image content (not borders) for bordered Lisa images.
