# Upload Processing (`content/upload.rs`)

## Summary

This module handles file uploads from HTTP multipart form data. It extracts files and metadata from multipart requests, validates MIME types against file extensions and magic bytes, and saves uploaded files to the temporary directory with UUID-based tokens.

## Imports

- `api::error::{ApiError, ApiResult}`: API error types.
- `config::Config`: Application configuration.
- `filesystem::{self, Directory}`: File system operations and directory constants.
- `model::enums::MimeType`: MIME type enum.
- `string::SmallString`: Small string optimization.
- `axum::extract::multipart::{Field, Multipart}`: Axum multipart extraction.
- `axum::extract::rejection::{JsonRejection, MissingJsonContentType}`: JSON rejection types.
- `serde::{Deserialize, Deserializer, Serialize}`: Serialization.
- `std::ffi::OsStr`: OS string handling.
- `std::path::{Path, PathBuf}`: File paths.
- `std::str::FromStr`: String parsing.
- `strum::IntoStaticStr`: Enum to static string conversion.
- `uuid::Uuid`: UUID generation.

## Constant: `MAX_UPLOAD_SIZE`

```rust
pub const MAX_UPLOAD_SIZE: usize = 4 * 1024_usize.pow(3);
```

Maximum upload size: 4 GB. Pow(3) means 1024^3 = 1 GB, times 4.

**This constant appears unused in this file.** It is likely referenced elsewhere in the upload pipeline.

## Struct: `UploadToken`

```rust
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct UploadToken {
    token: String,
    #[serde(skip)]
    mime_type: MimeType,
}
```

A token that represents an uploaded file saved to the temporary uploads directory. The `#[serde(transparent)]` attribute means it serializes as only the token string. The `mime_type` is `#[serde(skip)]` — it is not serialized.

**Fields:**
- `token: String`: UUID-based filename in the format `"{uuid}.{extension}"`.
- `mime_type: MimeType`: The MIME type of the uploaded file.

### Method: `UploadToken::new(mime_type) -> Self`

Creates a new token with a random UUID and the given MIME type.

**Algorithm:**
1. Generate token string: `format!("{}.{}", Uuid::new_v4(), mime_type.extension())`.
2. Store `token` and `mime_type` in the struct.

### Method: `UploadToken::mime_type() -> MimeType`

Returns the MIME type stored in the token.

### Method: `UploadToken::path(config: &Config) -> PathBuf`

Returns the full filesystem path to the temporary file.

**Algorithm:**
- Join the temporary uploads directory path with the token filename.

### Trait: `Deserialize<'de> for UploadToken`

Custom deserialization with path traversal protection.

**Algorithm:**
1. Deserialize as `String`.
2. **Security check:** If the token contains `/` or `\`, return error `"invalid upload token"`. This prevents directory traversal attacks.
3. Split on `.`: `(uuid_part, extension)`.
4. Parse the extension into `MimeType` via `MimeType::from_extension()`. Error on unknown extensions.
5. Return `UploadToken { token, mime_type }`.

## Enum: `PartName`

```rust
#[derive(Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum PartName {
    Content,
    Thumbnail,
    Avatar,
}
```

Names of expected multipart form fields. Each variant converts to a lowercase static string via `strum`:
- `Content` -> `"content"`
- `Thumbnail` -> `"thumbnail"`
- `Avatar` -> `"avatar"`

## Struct: `Body<const N: usize>`

```rust
pub struct Body<const N: usize> {
    pub files: [Option<UploadToken>; N],
    pub metadata: Option<Vec<u8>>,
}
```

Generic result of multipart extraction. Contains:
- `files`: An array of `N` optional `UploadToken`s, one per expected part.
- `metadata`: Optional raw JSON metadata bytes.

## Function: `extract<const N: usize>(config, form_data, fields) -> ApiResult<Body<N>>`

```rust
pub async fn extract<const N: usize>(
    config: &Config,
    mut form_data: Multipart,
    fields: [PartName; N],
) -> ApiResult<Body<N>>
```

Extracts files and metadata from a multipart form.

**Algorithm:**
1. Initialize `files = [None; N]` and `metadata = None`.
2. While there are more fields (`form_data.next_field().await`):
   - Try to find the field name in `fields` via `position()` where the field name matches the part name's static string.
   - If the field name matches neither `fields` nor `"metadata"`: skip the field.
   - **Get MIME type for file fields:** If `position` was found, call `get_mime_type(&field)`. This returns `(index, mime_type)`.
   - **Metadata validation:** If `position` is `None` (it is the metadata field) and the content type is not `"application/json"`: return an error (`ApiError::JsonRejection(MissingJsonContentType)`).
   - **Save file:** If `(index, mime_type)` exists, call `filesystem::save_uploaded_file(config, field, mime_type, None)` to stream the field to disk. Store the resulting `UploadToken` in `files[index]`.
   - **Read metadata:** If no position match, read the field bytes (`field.bytes().await`) and store as `metadata`.
3. Return `Body { files, metadata }`.

## Function: `get_mime_type(field: &Field) -> ApiResult<MimeType>`

Determines the MIME type of a multipart field, validating consistency between file extension and content type.

**Algorithm:**
1. Extract `extension` from the field's `file_name()` (get file extension).
2. Extract `content_type` from the field's `content_type()`.
3. Match on the combination:

| Extension | Content Type | Action |
|-----------|-------------|--------|
| Some | None or `application/octet-stream` | Parse MIME from extension only |
| Some | Some | Parse MIME from extension. If content type MIME differs, return `Err(ContentTypeMismatch)` |
| None | Some | Parse MIME from content type string |
| None | None | Return `Err(MissingContentType)` |

This ensures the file extension and HTTP content type header agree when both are present.

## Function: `verify_magic_bytes(path: &Path, mime_type: MimeType) -> ApiResult<()>`

Verifies that the magic bytes at the start of the file match the claimed MIME type. This prevents MIME type spoofing.

**Algorithm:**
1. Read up to 12 bytes from the file (`std::fs::read`).
2. Get the magic bytes slice: `&bytes[..min(12, bytes.len())]`.
3. Check against known magic byte patterns:

| MIME Type | Magic Bytes |
|-----------|-------------|
| `Png` | `starts_with([0x89, 0x50, 0x4E, 0x47])` ("PNG") |
| `Jpeg` | `starts_with([0xFF, 0xD8, 0xFF])` |
| `Gif` | `starts_with(b"GIF8")` (GIF87a or GIF89a) |
| `Bmp` | `starts_with(b"BM")` |
| `Webp` | `starts_with(b"RIFF")` AND `magic[8..12] == b"WEBP"` |
| `Mp4`, `Mov`, `Avif` | `magic[4..8] == b"ftyp"` (ISO base media file format) |
| `Webm` | `starts_with([0x1A, 0x45, 0xDF, 0xA3])` (EBML header) |
| `Swf` | `starts_with(b"FWS")` (uncompressed) or `starts_with(b"CWS")` (compressed) |
| `Pdf` | `starts_with(b"%PDF")` |

4. If valid: return `Ok(())`. Otherwise: return `Err(ApiError::InvalidMagicBytes(mime_type))`.

**Note:** `Mp4`, `Mov`, and `Avif` share the same magic byte check (`ftyp` box) because they all use the ISO base media file format container.
