# Content Cache (`content/cache.rs`)

## Summary

This module provides a ring buffer cache for computed content properties. When content is uploaded, expensive computations (decoding, checksums, signatures, thumbnails) are performed once and stored in the cache. A subsequent request for the same token retrieves cached results.

## Imports

- `api::error::ApiResult`: API result type.
- `content::hash::{Checksum, Md5Checksum}`: BLAKE3 and MD5 checksum types.
- `content::signature::COMPRESSED_SIGNATURE_LEN`: Length of compressed image signature.
- `content::thumbnail::ThumbnailType`: Thumbnail type enum.
- `content::upload::UploadToken`: Upload token.
- `content::{decode, hash, signature, thumbnail}`: Content processing functions.
- `extract::Ctx`: Request context.
- `model::enums::{MimeType, PostFlag, PostFlags, PostType}`: MIME types, flags, and post types.
- `content`, `filesystem`: Module-level imports.
- `image::DynamicImage`: Decoded image.
- `image::error::LimitErrorKind`: Image dimension error.
- `std::collections::VecDeque`: Ring buffer storage.

## Struct: `CachedProperties`

```rust
pub struct CachedProperties {
    pub token: UploadToken,
    pub checksum: Checksum,
    pub md5_checksum: Md5Checksum,
    pub signature: [i64; COMPRESSED_SIGNATURE_LEN],
    pub thumbnail: DynamicImage,
    pub width: i32,
    pub height: i32,
    pub mime_type: MimeType,
    pub post_type: PostType,
    pub file_size: i64,
    pub flags: PostFlags,
}
```

Stores all computed properties for a piece of content. Derives `Clone`.

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `token` | `UploadToken` | Identifier for the uploaded file |
| `checksum` | `Checksum` (32 bytes) | BLAKE3 checksum |
| `md5_checksum` | `Md5Checksum` (16 bytes) | MD5 checksum |
| `signature` | `[i64; COMPRESSED_SIGNATURE_LEN]` | Compressed image signature |
| `thumbnail` | `DynamicImage` | Generated thumbnail image |
| `width` | `i32` | Image width in pixels |
| `height` | `i32` | Image height in pixels |
| `mime_type` | `MimeType` | Content MIME type |
| `post_type` | `PostType` | Post type (image, video, etc.) |
| `file_size` | `i64` | File size in bytes |
| `flags` | `PostFlags` | Content flags (e.g., `Sound`) |

## Struct: `RingCache`

```rust
pub struct RingCache {
    data: VecDeque<(UploadToken, CachedProperties)>,
    max_size: usize,
}
```

A fixed-capacity ring buffer for `CachedProperties`. When the buffer is full, the oldest entry is evicted.

### Method: `RingCache::new(max_size) -> Self`

Creates a new empty `RingCache` with a maximum capacity of `max_size`.

### Method: `RingCache::clear(&mut self)`

Replaces the internal `VecDeque` with a new empty one. Evicts all cached entries.

### Method: `RingCache::insert(&mut self, key, value)`

Pushes a new `(key, value)` pair to the back of the deque. If the deque length exceeds `max_size`, calls `pop_front()` to evict the oldest entry.

**Parameters:**
- `key: UploadToken`: The token identifying the content.
- `value: CachedProperties`: The computed properties.

### Method: `RingCache::remove(&mut self, key) -> Option<CachedProperties>`

Finds and removes the entry matching `key`. Returns the `CachedProperties` if found, `None` otherwise.

**Algorithm:**
1. Search the deque for the position of `key` using `iter().position()`.
2. If found, call `remove(pos)` to extract the entry.
3. Map to extract only the `CachedProperties` (discard the key).

This is a "pop" operation — the entry is consumed from the cache.

## Function: `compute_properties(ctx, content_token) -> ApiResult<CachedProperties>`

```rust
pub fn compute_properties(ctx: &Ctx, content_token: UploadToken) -> ApiResult<CachedProperties>
```

Computes content properties and stores them in the cache.

**Algorithm:**
1. Call `compute_properties_no_cache(ctx, content_token.clone())` to compute properties.
2. Clone the result to avoid holding the cache lock too long.
3. Insert the properties into the cache: `ctx.get_content_cache().insert(content_token, properties_copy)`.
4. Return the original `properties`.

## Function: `get_or_compute_properties(ctx, content_token) -> ApiResult<CachedProperties>`

```rust
pub fn get_or_compute_properties(ctx: &Ctx, content_token: UploadToken) -> ApiResult<CachedProperties>
```

Returns cached properties if available, otherwise computes them.

**Algorithm:**
1. Try `ctx.get_content_cache().remove(&content_token)`.
2. If `Some(properties)`: return `Ok(properties)` (cache hit).
3. If `None`: call `compute_properties_no_cache(ctx, content_token)` (cache miss, compute fresh).

## Function: `compute_properties_no_cache(ctx, token) -> ApiResult<CachedProperties>`

```rust
fn compute_properties_no_cache(ctx: &Ctx, token: UploadToken) -> ApiResult<CachedProperties>
```

Computes all content properties without interacting with the cache. This is the core computation function.

**Algorithm:**
1. **File path:** `token.path(&ctx.config)`.
2. **File size:** `filesystem::file_size(&temp_path)`, wrapped in `map_read_result` (NotFound becomes InvalidUploadToken).
3. **Checksums:** `hash::compute_checksums(&temp_path)` returns `(checksum, md5_checksum)`.
4. **MIME type:** `token.mime_type()` (stored in the token).
5. **Post type:** `decode::detect_post_type(&temp_path, mime_type)`.
6. **Sound detection:**
   - `PostType::Image | Animation | Document`: `has_sound = false`.
   - `PostType::Video`: `decode::video_has_audio(&temp_path)`.
   - If `has_sound` is true, create `PostFlags::new_with(PostFlag::Sound)`. Otherwise `PostFlags::new()`.
7. **Image decoding:** `decode::representative_image(&temp_path, mime_type)`.
8. **Construct and return `CachedProperties`:**
   - `token`: The input token.
   - `checksum`: BLAKE3.
   - `md5_checksum`: MD5.
   - `signature`: `signature::compute(&image)`.
   - `thumbnail`: `thumbnail::create(&ctx.config, &image, ThumbnailType::Post)`.
   - `width`: `i32::try_from(image.width())`, error if exceeds i32 range (maps to DimensionError).
   - `height`: `i32::try_from(image.height())`, same conversion.
   - `mime_type`: From token.
   - `post_type`: Detected post type.
   - `file_size`: File size in bytes.
   - `flags`: Sound flag set or empty.
