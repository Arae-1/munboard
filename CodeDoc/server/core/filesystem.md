# filesystem.rs — File System Operations

## Purpose

This file defines data directory types and provides file system operations for saving, deleting, moving, and swapping media files, thumbnails, avatars, and temporary uploads.

## Constant: `MAX_URL_DOWNLOAD_SIZE`

```rust
pub const MAX_URL_DOWNLOAD_SIZE: usize = 4 * 1024_usize.pow(3);
```

Maximum allowed size for URL-based downloads. Set to 4 GiB.

## Enum: `Directory`

| Variant | String | Purpose |
|---------|--------|---------|
| `Avatars` | `avatars` | Custom user avatars |
| `Posts` | `posts` | Post content files |
| `GeneratedThumbnails` | `generated-thumbnails` | Auto-generated post thumbnails |
| `CustomThumbnails` | `custom-thumbnails` | User-customized post thumbnails |
| `TemporaryUploads` | `temporary-uploads` | Staged upload files awaiting processing |

Derives `Clone`, `Copy`, `Display`, `IntoStaticStr`. Display serializes to `kebab-case`.

## Public Functions

### `file_size(path: &Path) -> std::io::Result<i64>`

Returns the file size at `path` in bytes as `i64`. Panics if the size exceeds `i64::MAX`.

### `save_uploaded_file<S, E>(config, stream, mime_type, max_size) -> ApiResult<UploadToken>`

Saves a streamed file to the temporary uploads directory.

**Parameters:**
- `config: &Config` — Server configuration
- `stream: S` — An async stream yielding `Result<Bytes, E>` chunks
- `mime_type: MimeType` — The MIME type of the uploaded file
- `max_size: Option<usize>` — Optional size limit

**Process:**
1. Creates an `UploadToken` with the MIME type.
2. Creates parent directories for the upload path.
3. Streams chunks to the file, tracking total bytes.
4. If `max_size` is set and exceeded, returns `ApiError::DownloadTooLarge(max)`.
5. Verifies magic bytes after saving.
6. Returns the `UploadToken`.

**Error handling:** Does not clean up on error. Stale uploads are cleaned by the periodic cleanup task.

### `save_custom_avatar(config, username, thumbnail) -> ImageResult<i64>`

Saves a custom avatar thumbnail for a user.

**Parameters:**
- `config: &Config`
- `username: &str`
- `thumbnail: &DynamicImage`

**Process:**
1. Gets the avatar path from config.
2. Creates parent directories.
3. Converts image to RGB8 and saves.
4. Returns the file size in bytes.

### `delete_custom_avatar(config, username) -> std::io::Result<()>`

Deletes a user's custom avatar from disk if it exists. Uses `remove_if_exists` so missing files are not an error.

### `save_post_thumbnail(post, thumbnail, thumbnail_type) -> ImageResult<i64>`

Saves a post thumbnail (generated or custom) to disk.

**Parameters:**
- `post: &PostHash` — The post identifier
- `thumbnail: &DynamicImage` — The thumbnail image
- `thumbnail_type: ThumbnailCategory` — Generated or Custom

**Process:**
1. Gets the thumbnail path based on type.
2. Creates parent directories.
3. Converts to RGB8 and saves.
4. Returns file size.

### `delete_post_thumbnail(post, thumbnail_type) -> std::io::Result<()>`

Deletes a post thumbnail if it exists. Non-existent files are not an error.

### `delete_content(post, mime_type) -> std::io::Result<()>`

Deletes a post's content file from disk.

### `delete_post(post, mime_type) -> std::io::Result<()>`

Deletes all files for a post: generated thumbnail, custom thumbnail, and content.

### `swap_posts(config, post_a, mime_type_a, post_b, mime_type_b) -> std::io::Result<()>`

Swaps the filesystem contents of two posts as if their IDs were swapped.

**Process:**
1. Swaps generated thumbnails (always exist, always `.jpg`).
2. Handles four cases for custom thumbnails (both exist, only A exists, only B exists, neither exists).
3. Handles two cases for content files (same MIME type, different MIME types).

### `move_file(from, to) -> std::io::Result<()>`

Moves a file from `from` to `to`.

**Process:**
1. Creates parent directories of `to`.
2. Tries `std::fs::rename` first.
3. If rename fails with `CrossesDevices`, falls back to copy-then-remove.
4. Sets standard permissions (`0o644`) on the destination. Warns if permission setting fails.

### `purge_temporary_uploads(config) -> std::io::Result<()>`

Deletes all files in the temporary uploads directory. Used at startup to clean leftover uploads from a previous run.

### `spawn_temporary_uploads_cleanup_task(config: Arc<Config>)`

Spawns an async background task that runs every 10 minutes (`CLEANUP_INTERVAL`). The task:

1. Collects current uploads from the temp directory.
2. On the next tick, deletes any uploads that were present in the previous interval (two-pass stale detection).
3. On any error, resets the set and logs a warning.

## Private Functions

### `remove_if_exists(file: &Path) -> std::io::Result<()>`

Removes a file if it exists. Returns `Ok(())` if the file does not exist. Only returns an error for permission issues or other non-NotFound errors.

### `remove_stale_uploads(config, stale_uploads) -> std::io::Result<HashSet<PathBuf>>`

Deletes files from the temp directory that appear in `stale_uploads`. Returns the set of files currently in the temp directory.

### `swap_files(config, file_a, file_b) -> std::io::Result<()>`

Swaps two files by temporarily renaming one to a temp path, moving the second to the first's path, then moving the temp to the second's path.

### `set_permissions(path: &Path) -> std::io::Result<()>`

Sets standard file permissions (`0o644`) on a path. Prevents permission errors on different systems after moving files.

### `create_parent_directories(path: &Path) -> std::io::Result<()>`

Recursively creates all parent directories for a path. Does not error if directories already exist.
