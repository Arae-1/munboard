# Content Processing (`content/mod.rs`)

## Summary

This module is the top-level content processing module. It defines the `Content` enum for representing content sources (upload tokens or URLs), and provides methods to save, thumbnail, and compute properties for content. It re-exports all submodules.

## Imports

- `api::error::{ApiError, ApiResult}`: API error types.
- `config::Config`: Application configuration.
- `content::cache::CachedProperties`: Computed content properties stored in cache.
- `content::thumbnail::ThumbnailType`: Post or avatar thumbnail.
- `content::upload::UploadToken`: Token for a file in the temporary uploads directory.
- `extract::Ctx`: Request context.
- `image::DynamicImage`: Decoded image.
- `url::Url`: URL type.

## Module Declarations

All modules are public:

| Module | Visibility | Purpose |
|--------|-----------|---------|
| `cache` | `pub` | Ring buffer cache for content properties |
| `decode` | `pub` | Image/video decoding |
| `download` | `pub` | HTTP content download |
| `hash` | `pub` | Content hashing |
| `signature` | `pub` | Image signature computation |
| `thumbnail` | `pub` | Thumbnail generation |
| `upload` | `pub` | Upload processing pipeline |

## Enum: `Content`

```rust
pub enum Content {
    Token(UploadToken),
    Url(Url),
}
```

Represents the source of content to process. Either:
- `Token(UploadToken)`: A file already uploaded to the temporary uploads directory (identified by a token).
- `Url(Url)`: A URL pointing to a file on the web to download.

### Method: `Content::new(token, url) -> Option<Self>`

Constructs a `Content` from optional token and URL.

**Priority rules:**
1. If `token` is `Some`: returns `Content::Token(token)`.
2. Else if `url` is `Some`: returns `Content::Url(url)`.
3. Else: returns `None` (no content to process).

### Method: `Content::save(self, config) -> ApiResult<UploadToken>`

Ensures content is saved to the temporary uploads directory. Consumes `self`.

**Algorithm:**
- If `Content::Token(token)`: Return `Ok(token)` immediately (already on disk).
- If `Content::Url(url)`: Call `download::from_url(config, url).await` to download and save.

Returns an `UploadToken` that points to the saved file.

### Method: `Content::thumbnail(self, config, thumbnail_type) -> ApiResult<DynamicImage>`

Computes a thumbnail for the content. Consumes `self`.

**Algorithm:**
1. Call `self.save(config).await` to ensure content is on disk. Get the `token`.
2. Construct the file path: `token.path(config)`.
3. Use `tokio::task::block_in_place` to run CPU-bound work without blocking the async runtime:
   - Call `decode::representative_image(&temp_path, token.mime_type())` to get a decoded image.
   - Call `thumbnail::create(config, &image, thumbnail_type)` to generate the thumbnail.
4. Return the `DynamicImage` thumbnail.

### Method: `Content::compute_properties(self, ctx) -> ApiResult<CachedProperties>`

Computes all content properties. Consumes `self`.

**Algorithm:**
1. Call `self.save(&ctx.config).await` to get a token.
2. Use `tokio::task::block_in_place` to call `cache::compute_properties(ctx, token)`.

Returns a `CachedProperties` containing checksums, signature, thumbnail, dimensions, MIME type, post type, file size, and flags.

### Method: `Content::get_or_compute_properties(self, ctx) -> ApiResult<CachedProperties>`

Gets cached properties or computes them if not cached. Consumes `self`.

**Algorithm:**
1. Call `self.save(&ctx.config).await` to get a token.
2. Use `tokio::task::block_in_place` to call `cache::get_or_compute_properties(ctx, token)`.

This avoids recomputing properties if the same content was recently processed (ring buffer cache).

## Function: `map_read_result<T>(result: std::io::Result<T>) -> ApiResult<T>`

```rust
fn map_read_result<T>(result: std::io::Result<T>) -> ApiResult<T>
```

Maps an I/O `Result` to an `ApiResult`, converting `NotFound` errors to `ApiError::InvalidUploadToken` and all others to `ApiError::from(err)`.

This is used internally to distinguish "file was removed" from other I/O errors.
