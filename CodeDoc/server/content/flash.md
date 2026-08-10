# Flash Processing (`content/flash.rs`)

## Summary

This module decodes images from Adobe Flash (SWF) files. The code is based on Ruffle (<https://github.com/ruffle-rs/ruffle>), the open-source Flash emulator, licensed under MIT. It handles the various bitmap formats used in SWF: plain JPEG in `DefineBits`, JPEG with alpha in `DefineBitsJPEG3`, and lossless formats in `DefineBitsLossless`.

## License

MIT License, copyright (c) 2018-2022 Ruffle LLC and Ruffle contributors.

## Imports

- `flate2::bufread::ZlibDecoder`: Zlib decompression.
- `image::{DynamicImage, ImageFormat, ImageReader, Limits, RgbaImage}`: Image types and decode limits.
- `std::borrow::Cow`: Copy-on-write byte slices.
- `std::io::{Cursor, Read}`: In-memory cursor and read trait.
- `swf::Color`: SWF color type.
- `tracing::{error, warn}`: Logging.

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_EMBEDDED_IMAGE_DIMENSION` | `16384` (16K) | Maximum width/height, in pixels, of images embedded in SWF tags. Larger images are rejected before the decoder allocates any buffers. |
| `MAX_EMBEDDED_IMAGE_ALLOCATION` | `0x1000_0000` (256 MiB) | Maximum number of bytes image decoders may allocate for images embedded in SWF tags. Prevents multi-GB allocations from embedded image bombs. |
| `MAX_BITMAP_AREA` | `0x0200_0000` (32M pixels) | Maximum area, in pixels, of a `DefineBitsLossless` bitmap. The worst-case RGBA output is `MAX_BITMAP_AREA * 4` bytes, which keeps the per-tag memory (decompressed payload plus output) at or below 256 MiB. |
| `MAX_DECOMPRESSED_SIZE` | `0x0800_0000` (128 MiB) | Maximum number of bytes decompressed from a single zlib payload. |

## Enum: `Error`

```rust
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    #[error("Bitmap texture is larger than the rendering device supports")]
    TooLarge,
    #[error("Unknown bitmap format")]
    UnknownType,
    #[error("Invalid ZLIB compression")]
    InvalidZlibCompression,
    Image(#[from] image::ImageError),
    #[error("Unsupported DefineBitsLossless{0} format {1:?}")]
    UnsupportedLosslessFormat(u8, swf::BitmapFormat),
}
```

Error types for Flash decoding. Derives `#[error(transparent)]` for the source chain.

## Enum: `JpegTagFormat`

```rust
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JpegTagFormat {
    Jpeg,
    Png,
    Gif,
    Unknown,
}
```

The image format inside a `DefineBitsJpeg2` or `DefineBitsJpeg3` tag. According to SWF19 pp.138-139, these tags can contain JPEG, PNG, or GIF data (not only JPEG as the name suggests).

## Function: `determine_jpeg_tag_format(data: &[u8]) -> JpegTagFormat`

Determines the image format from magic bytes.

**Algorithm:**
| Magic Bytes | Format |
|-------------|--------|
| `[0xFF, 0xD9, 0xFF, 0xD8, ...]` or `[0xFF, 0xD8, ...]` | `Jpeg` (first includes erroneous SWF header) |
| `[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, ...]` | `Png` (PNG signature) |
| `[0x47, 0x49, 0x46, 0x38, 0x37\|0x39, 0x61, ...]` | `Gif` (GIF87a or GIF89a) |
| Other | `Unknown` |

## Function: `decode_define_bits_jpeg(data, alpha_data) -> Result<DynamicImage, Error>`

Decodes bitmap data from a `DefineBitsJPEG2` or `DefineBitsJPEG3` tag.

**Algorithm:**
1. Call `determine_jpeg_tag_format(data)`.
2. Validate: if format is not JPEG and `alpha_data` is `Some`, log a warning (only true JPEG with DefineBitsJPEG3 should have separate alpha).
3. Dispatch by format:
   - `JpegTagFormat::Jpeg`: Call `decode_jpeg(data, alpha_data)`.
   - `JpegTagFormat::Png`: Decode as PNG via `decode_embedded_image(data, ImageFormat::Png)`.
   - `JpegTagFormat::Gif`: Decode as GIF via `decode_embedded_image(data, ImageFormat::Gif)`.
   - `JpegTagFormat::Unknown`: Return `Err(Error::UnknownType)`.

## Function: `decode_embedded_image(data, format) -> Result<DynamicImage, Error>`

Decodes image data embedded in an SWF tag with the same size limits applied to regular image uploads, so an embedded PNG/GIF bomb cannot force a multi-GB allocation.

**Algorithm:**
1. Create an `ImageReader` over a `Cursor` of the data.
2. Set the format explicitly.
3. Set decoder limits to `embedded_image_limits()`.
4. Call `reader.decode()`, mapping errors to `Error`.

## Function: `embedded_image_limits() -> Limits`

Returns the decode limits applied to images embedded in SWF tags:
- `max_image_width = Some(MAX_EMBEDDED_IMAGE_DIMENSION)` (16384).
- `max_image_height = Some(MAX_EMBEDDED_IMAGE_DIMENSION)` (16384).
- `max_alloc = Some(MAX_EMBEDDED_IMAGE_ALLOCATION)` (256 MiB).

All other limits are unbounded (`Limits::no_limits()`).

## Function: `glue_tables_to_jpeg(jpeg_data, jpeg_tables) -> Cow<[u8]>`

Joins JPEG encoding tables (from a `JPEGTables` SWF tag) with JPEG data (from a `DefineBits` tag).

**Algorithm:**
1. If `jpeg_tables` is `Some` and has length >= 2:
   - Allocate a new `Vec` with capacity for both slices.
   - Append `jpeg_tables[..len-2]` (strip the trailing EOI marker from tables).
   - If `jpeg_data` has length >= 2: append `jpeg_data[2..]` (strip the leading SOI marker).
   - Return `Cow::Owned(full_jpeg)`.
2. Otherwise: return `Cow::Borrowed(jpeg_data)` (no tables to glue).

## Function: `remove_invalid_jpeg_data(data: &[u8]) -> Cow<[u8]>`

Removes invalid JPEG marker sequences from SWF `DefineBits*` tags.

**Background:** SWF files can contain an interior `0xFFD9FFD8` (EOI+SOI) marker sequence. The Flash decoder ignores this, but standard JPEG decoders stop at EOI. This function removes these sequences.

**Algorithm:**
1. Constants: `SOF0=0xC0`, `RST0..RST7=0xD0..0xD7`, `SOI=0xD8`, `EOI=0xD9`.
2. Try stripping leading `[0xFF, EOI, 0xFF, SOI]` prefix (common case, zero-copy via `Cow::Borrowed`).
3. If not found at start: parse JPEG markers sequentially.
   - For each 4-byte window:
     - `[0xFF, EOI, 0xFF, SOI]`: Found invalid sequence. Create new `Vec` with bytes before and after the sequence spliced out. Return `Cow::Owned`.
     - `[0xFF, EOI|SOI|RST0..RST7, _, _]`: Marker without size (EOI, SOI, RST). Advance 2 bytes.
     - `[0xFF, SOF0, _, _]`: Start of frame reached without finding invalid sequence. Return `Cow::Borrowed`.
     - `[0xFF, _, a, b]`: Other marker with length (`u16::from_be_bytes([a, b])`). Advance by length + 2.
     - Other: Not a valid JPEG marker start. Return `Cow::Borrowed`.
4. After cleaning: check if data ends with `[0xFF, EOI]`. If not, append one (warn about missing EOI marker). Return data.

## Function: `validate_size(width: u16, height: u16) -> Result<(), Error>`

Checks that bitmap dimensions are reasonable.

**Algorithm:**
- Compute `size = (width as usize).saturating_mul(height as usize)`.
- If `size >= MAX_BITMAP_AREA` (32M pixels, about 128 MiB of RGBA output): return `Err(Error::TooLarge)`.
- Otherwise: `Ok(())`.

## Function: `decode_jpeg(jpeg_data, alpha_data) -> Result<DynamicImage, Error>`

Decodes JPEG data, optionally with separate alpha channel.

**Algorithm:**
1. Call `remove_invalid_jpeg_data(jpeg_data)` to clean the JPEG data.
2. Decode JPEG: `decode_embedded_image(&jpeg_data, ImageFormat::Jpeg)` (applies the embedded-image limits).
3. If `alpha_data` is `Some`:
   - Get RGB bytes: `jpeg.as_bytes()` (RGB layout, 3 bytes per pixel).
   - Decompress alpha: `decompress_zlib(alpha_data)`.
   - If alpha length matches `decoded_data.len() / 3` (one alpha byte per pixel):
     - Interleave RGB with alpha into RGBA.
     - **Clamp colors to alpha:** For each pixel, clamp `r = rgb[0].min(a)`, `g = rgb[1].min(a)`, `b = rgb[2].min(a)`. This fixes SWFs where premultiplied alpha incorrectly has color on fully transparent pixels. Flash Player does this clamping.
     - Try `RgbaImage::from_vec(width, height, rgba)`. Return if successful.
   - If alpha size mismatch: log error, fall back to RGB-only.
4. Return the decoded JPEG (possibly with alpha).

## Function: `decode_define_bits_lossless(swf_tag: &swf::DefineBitsLossless) -> Result<Option<DynamicImage>, Error>`

Decodes bitmap data from a `DefineBitsLossless` tag. These are Zlib-compressed pixel data, possibly palettized.

**Algorithm:**
1. Validate the bitmap dimensions: call `validate_size(swf_tag.width, swf_tag.height)`. Reject oversized bitmaps before decompressing anything, so oversized tags are never decompressed.
2. `has_alpha` is true for version 2, false for version 1.
3. Decompress the tag data: `decompress_zlib(&swf_tag.data)`.
4. Match on `(version, format)`:

**Version 1, `BitmapFormat::Rgb15`:**
- Padded width: `(width + 1) & !1` (round to even).
- Truncated-payload check: the payload must hold the full padded bitmap (`expected_len = padded_width * height * 2` bytes). If `decoded_data.len() < expected_len`: return `Err(Error::InvalidZlibCompression)`. This prevents indexing out of bounds below.
- Allocate output buffer: `width * height * 4` bytes (RGBA).
- For each pixel: read 2 bytes (big-endian u16). Extract 5-bit RGB components (shift 10, 5, 0), scale to 8-bit: `(component * 255 + 15) / 31`. Set alpha to 255.
- Skip padding bytes at end of each row.

**Version 1 or 2, `BitmapFormat::Rgb32`:**
- Data is already RGBA (in ARGB byte order in SWF). Rotate channels left by 1: `rotate_left(1)` converts ARGB to RGBA.
- If `!has_alpha`: force alpha channel to 255.

**Version 1 or 2, `BitmapFormat::ColorMap8 { num_colors }`:**
- Padded width: `(width + 3) & !3` (round to multiple of 4).
- Truncated-payload check: the payload must hold the palette plus the full padded bitmap. `palette_len = (num_colors + 1) * (4 if has_alpha else 3)`. If `decoded_data.len() < palette_len + padded_width * height`: return `Err(Error::InvalidZlibCompression)`. This prevents indexing out of bounds below.
- Read palette: `num_colors + 1` entries. Each entry is 3 bytes (RGB) for version 1, 4 bytes (RGBA) for version 2. Alpha defaults to 255 for version 1.
- For each pixel (1-byte index): look up palette entry, write RGBA to output.
- Skip padding bytes at end of each row.

Other formats: return `Err(UnsupportedLosslessFormat)`.

5. Construct `RgbaImage` from output data, convert to `DynamicImage`. Return `None` if `from_vec` fails.

## Function: `decompress_zlib(data: &[u8]) -> Result<Vec<u8>, Error>`

Decompresses Zlib-compressed data with a size limit.

**Algorithm:**
1. Create a `ZlibDecoder` from `data`.
2. Limit read to `MAX_DECOMPRESSED_SIZE` (128 MiB) via `.take()`.
3. Read all bytes into `out_data`.
4. Use `.shrink_to_fit()` to release excess capacity.
5. Return `out_data`. Map decompression errors to `InvalidZlibCompression`.

## Constant: `MAX_DECOMPRESSED_SIZE`

```rust
const MAX_DECOMPRESSED_SIZE: u64 = 0x0800_0000; // 128 MiB
```

Maximum size of decompressed Zlib data. Prevents memory exhaustion attacks.
