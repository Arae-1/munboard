# Content Hashing (`content/hash.rs`)

## Summary

This module provides content hashing with BLAKE3 (primary, for duplicate detection) and MD5 (secondary, for external compatibility). It also defines `PostHash` for constructing disk paths and URLs for post content, and `GenericChecksum` for fixed-size checksum storage with database serialization.

## Imports

- `config::Config`: Application configuration.
- `filesystem::Directory`: Directory path enum.
- `model::enums::MimeType`: MIME type enum.
- `base64::Engine` and `base64::engine::general_purpose::URL_SAFE_NO_PAD`: URL-safe Base64 encoding.
- `blake3::KEY_LEN`: BLAKE3 key length constant.
- `diesel::*`: PostgreSQL serialization for `GenericChecksum`.
- `hex::{FromHex, FromHexError}`: Hex encoding/decoding.
- `std::fmt::Display`: Display trait.
- `std::fs::File`: File I/O.
- `std::io::Read`: Read trait.
- `std::path::{Path, PathBuf}`: File paths.
- `std::str::FromStr`: String parsing.

## Struct: `PostHash<'a>`

```rust
pub struct PostHash<'a> {
    post_id: i64,
    hash: String,
    config: &'a Config,
}
```

Stores a post ID and its derived content hash. The hash is a keyed BLAKE3 of the post ID using `content_secret` from the configuration. This provides obfuscated file names so that post files cannot be guessed by ID.

### Method: `PostHash::new(config, post_id) -> Self`

Constructs a `PostHash`.

**Algorithm:**
1. Derive a 32-byte key from `config.content_secret`: pad to `KEY_LEN` bytes with zeros.
2. Compute `blake3::keyed_hash(&key, &post_id.to_le_bytes())`.
3. Encode the hash bytes as URL-safe Base64 without padding.
4. Store `hash`, `post_id`, and `config`.

### Method: `PostHash::id() -> i64`

Returns the post ID.

### Method: `PostHash::content_url(content_type) -> String`

Returns the URL to post content. Format: `{data_url}/{POSTS_DIRECTORY}/{self}.{extension}`.

The `Display` impl for `PostHash` produces the directory structure: `{outer_bucket}/{inner_bucket}/{post_id}_{hash}`.

### Method: `PostHash::thumbnail_url() -> String`

Returns the URL to the post thumbnail. Checks if a custom thumbnail exists on disk. If yes, uses `CustomThumbnails` directory. If no, uses `GeneratedThumbnails`.

**Algorithm:**
1. Check `self.custom_thumbnail_path().exists()`.
2. Set `thumbnail_folder` to the appropriate directory.
3. Return `"{data_url}/{thumbnail_folder}/{self}.{THUMBNAIL_EXTENSION}"` (extension is `"jpg"`).

### Method: `PostHash::content_path(content_type) -> PathBuf`

Returns the filesystem path to post content: `{posts_directory}/{self}.{extension}`.

### Method: `PostHash::generated_thumbnail_path() -> PathBuf`

Returns path to generated thumbnail: `{generated_thumbnails_directory}/{self}.{THUMBNAIL_EXTENSION}`.

### Method: `PostHash::custom_thumbnail_path() -> PathBuf`

Returns path to custom thumbnail: `{custom_thumbnails_directory}/{self}.{THUMBNAIL_EXTENSION}`.

## Trait Implementation: `Display for PostHash`

Formats the post hash as a directory path string.

**Algorithm:**
- `outer_bucket = post_id / 1_000_000` (groups of 1,000,000 posts)
- `inner_bucket = (post_id / 10_000) % 100` (groups of 10,000, modulo 100)
- Format: `"{outer_bucket:06}/{inner_bucket:02}/{post_id}_{hash}"`

This creates a 3-level directory structure to distribute files evenly.

## Type Aliases

```rust
pub type Checksum = GenericChecksum<32>;    // BLAKE3 (32 bytes)
pub type Md5Checksum = GenericChecksum<16>;  // MD5 (16 bytes)
```

## Struct: `GenericChecksum<const N: usize>`

```rust
#[derive(Debug, Clone, PartialEq, Eq, AsExpression, FromSqlRow)]
#[diesel(sql_type = Bytea)]
pub struct GenericChecksum<const N: usize>([u8; N]);
```

A fixed-size byte array that represents a checksum. Implements Diesel `Bytea` serialization for PostgreSQL storage.

### Method: `GenericChecksum::from_bytes(bytes: &[u8]) -> Self`

Constructs a checksum from a byte slice. Copies up to `N` bytes. Remaining bytes are zeroed.

**Algorithm:**
1. Create `[0; N]` array.
2. Copy `min(bytes.len(), N)` bytes from the input.
3. Return `Self(checksum)`.

This is a `const fn`.

### Trait: `AsRef<[u8]> for GenericChecksum<N>`

Returns `self.0.as_slice()`.

### Trait: `From<[u8; N]> for GenericChecksum<N>`

Constructs directly from a fixed-size array.

### Trait: `FromStr for GenericChecksum<N>`

Where `[u8; N]: FromHex`. Parses a hex-encoded string into a `GenericChecksum`.

### Trait: `ToSql<Bytea, Pg> for GenericChecksum<N>`

Serializes to PostgreSQL `Bytea` by delegating to `[u8]`'s implementation.

### Trait: `FromSql<Bytea, Pg> for GenericChecksum<N>`

Deserializes from PostgreSQL `Bytea`. Reads `value.as_bytes()` and calls `Self::from_bytes()`. This allows reading truncated data (e.g., 16-byte MD5 stored in a 32-byte column) by zero-padding.

## Function: `gravatar_url(config, username) -> String`

Generates a Gravatar URL for a user's avatar.

**Algorithm:**
1. Lowercase the username.
2. Compute `blake3::hash(username_lowercase.as_bytes())`.
3. Hex-encode the hash bytes.
4. Return `"https://gravatar.com/avatar/{hex_hash}?d=retro&s={avatar_width}"`.

## Function: `compute_checksums(path) -> std::io::Result<(Checksum, Md5Checksum)>`

Computes BLAKE3 and MD5 checksums for a file in a single pass.

**Algorithm:**
1. Define buffer size: 4 MB (`4 * 1024 * 1024`).
2. Open the file.
3. Create a `blake3::Hasher` and an `md5::Context`.
4. Allocate a 4 MB buffer.
5. Loop reading the file into the buffer:
   - If `n == 0`: break (EOF).
   - Update BLAKE3 hasher with `&buffer[..n]`.
   - Update MD5 context with `&buffer[..n]`.
6. Finalize: `blake3_hasher.finalize()` -> 32-byte array. `md5_ctx.compute()` -> 16-byte digest.
7. Wrap both in `GenericChecksum` and return as tuple.

## Function: `compute_url_safe_hash(content: &[u8]) -> String`

Computes a URL-safe BLAKE3 hash of arbitrary content. Returns Base64-encoded (no padding) string.

**Constant:**
```rust
const THUMBNAIL_EXTENSION: &str = "jpg";
```

All thumbnails use JPEG format.
