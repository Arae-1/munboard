# Content Processing

## Individual Files

- [mod.md](mod.md)
- [cache.md](cache.md)
- [decode.md](decode.md)
- [download.md](download.md)
- [embed.md](embed.md)
- [flash.md](flash.md)
- [hash.md](hash.md)
- [signature.md](signature.md)
- [thumbnail.md](thumbnail.md)
- [upload.md](upload.md)

## Purpose

The content module handles the full upload and processing pipeline for post content. It manages downloading, decoding, hashing, thumbnail generation, Flash processing, signature extraction, and caching.

The embed page documents the OpenGraph embed pages served for post links. The embed module renders post previews for chat applications and crawlers.

## Upload Pipeline Overview

1. Content arrives as either an `UploadToken` (already saved to the temporary uploads directory) or a `Url` (to be downloaded).
2. Content is saved to disk in the temporary uploads directory.
3. The file is decoded to produce a representative image.
4. Checksums (BLAKE3 and MD5) are computed from the file.
5. A post type is detected (image, animation, video, flash, document).
6. Audio presence is checked for videos and Flash files.
7. An image signature is computed for similarity search.
8. A thumbnail is generated.
9. Properties are cached in memory to avoid recomputation.

## Content Enum (`mod.rs`)

### `Content`
An enum with two variants:
- `Token(UploadToken)`: Content already saved to the temporary uploads directory.
- `Url(Url)`: A URL to download content from.

### Methods
- `new(token, url)`: Constructs a Content. Prefers a token over a URL.
- `save(config)`: Saves content to disk. Returns the UploadToken. Downloads from URL if needed.
- `thumbnail(config, thumbnail_type)`: Saves the content, decodes a representative image, and creates a thumbnail.
- `compute_properties(ctx)`: Saves content, then computes all properties and caches them in the ring buffer.
- `get_or_compute_properties(ctx)`: Tries to get properties from cache first; computes them if not cached.

### `map_read_result`
Maps filesystem errors. If a file is not found, returns `ApiError::InvalidUploadToken` instead of a generic IO error.

## Cache (`cache.rs`)

### `CachedProperties`
Stores all computed properties for a piece of content:
- `token`: The upload token identifying the file.
- `checksum`: BLAKE3 checksum (32 bytes).
- `md5_checksum`: MD5 checksum (16 bytes).
- `signature`: Compressed image similarity signature.
- `thumbnail`: The generated thumbnail image.
- `width`, `height`: Image dimensions.
- `mime_type`: The MIME type of the content.
- `post_type`: Detected post type (image, video, flash, etc.).
- `file_size`: Size in bytes.
- `flags`: Post flags (e.g., has sound).

### `RingCache`
A simple ring buffer (FIFO deque) that stores CachedProperties up to a maximum size. Oldest entries are evicted when the buffer is full.

### `compute_properties`
Computes properties without cache and then inserts them into the cache.

### `get_or_compute_properties`
Removes and returns cached properties if present. Falls back to computing without cache if not found.

### `compute_properties_no_cache`
The full property computation without cache interaction:
1. Gets file size from the filesystem.
2. Computes BLAKE3 and MD5 checksums in a single pass.
3. Determines MIME type from the upload token.
4. Detects post type (image, video, animation, flash, document).
5. Checks for audio in videos and SWF files.
6. Sets the Sound flag if audio is present.
7. Decodes a representative image.
8. Computes the image signature.
9. Generates a thumbnail.
10. Gets image dimensions.

## Decode (`decode.rs`)

### `representative_image`
Returns a representative image for the given content based on MIME type:
- **BMP, GIF, JPEG, PNG, WebP**: Decoded with the `image` crate.
- **SWF**: Searches Flash tags for the largest decodable image. Falls back to FFmpeg for video frames.
- **AVIF**: Decoded via FFmpeg as an RGBA image.
- **PDF**: Rendered via `pdftoppm` (Poppler) as a PNG.
- **MOV, MP4, WebM**: Uses FFmpeg's thumbnail filter to extract a representative video frame.

### `video_has_audio`
Uses FFmpeg to check if a video file contains audio streams. Streams the first 0 seconds (no actual decode) and checks for parsed audio stream events.

### `swf_has_audio`
Parses the SWF file and checks for any sound-related tags: DefineButtonSound, DefineSound, SoundStreamBlock, SoundStreamHead, SoundStreamHead2, StartSound, StartSound2.

### `detect_post_type`
Determines the post type from a file:
- **AVIF**: Checks if animated using FFmpeg (extracts frames from each video stream).
- **GIF**: Decodes the first two frames. If a second frame exists, it is animated.
- **WebP**: Uses the decoder's `has_animation()` method.
- **BMP, JPEG, PNG**: Always image.
- **MP4, MOV, WebM**: Always video.
- **SWF**: Always flash.
- **PDF**: Always document.

### `ffmpeg_frame`
Uses FFmpeg to extract a single frame from a file:
- For video/flash: Uses the `thumbnail` filter with RGB24 format. Limits to first 30 seconds.
- For images: Uses RGBA format.
- Returns a single `OutputFrame` event's data as a DynamicImage.

### `pdf_representative_image`
Renders the first page of a PDF to PNG using `pdftoppm` at 100 DPI. Falls back to a neutral 1x1 pixel image on any error.

### `flash_image`
Searches SWF tags for decodable images:
- **DefineBits**: JPEG with optional encoding tables from JpegTables tag.
- **DefineBitsLossless**: Zlib-compressed bitmap (palettized or raw RGB).
- **DefineBitsJpeg2/3**: JPEG data, possibly with PNG/GIF data or alpha channel.
- Falls back to FFmpeg frame extraction if no images found.
- Sorts images by effective thumbnail width for best quality selection.

### Animation Detection
- `avif_is_animated`: Uses FFmpeg to extract 2 frames from each video stream. Returns true if more than one frame is produced.
- `gif_is_animated`: Opens the GIF decoder and checks for a second frame.
- `webp_is_animated`: Uses the decoder's built-in `has_animation()`.

### `image_reader_limits`
Returns image reading limits: maximum dimension 16384 px, maximum allocation 512 MiB. The limits prevent memory exhaustion attacks.

## Download (`download.rs`)

### `from_url`
Downloads content from a URL:
1. Validates the URL: only HTTP and HTTPS schemes are allowed.
2. Resolves the host and rejects the URL if any resolved IP is private. The resolved addresses are pinned with `resolve_to_addrs` to prevent DNS rebinding.
3. Sends a GET request with a fake user agent and a 30-second timeout.
4. Follows redirects manually, up to 10. Each redirect is validated against the private-IP filter.
5. On 403 Forbidden, retries once with the original URL as the Referer header.
6. Extracts the Content-Type header to determine the MIME type.
7. Streams the response body to a file in the temporary uploads directory, limited to `config.max_url_download_size`.

### `is_private_ip`
Checks if an IP address is private or otherwise unreachable from the public internet:
- IPv4: 0.0.0.0/8, 10.0.0.0/8, 127.0.0.0/8, 169.254.0.0/16, 172.16.0.0/12, 192.168.0.0/16, 100.64.0.0/10 (CGNAT), 192.0.0.0/24, 198.18.0.0/15 (benchmarking), 224.0.0.0/4 (multicast).
- IPv6: loopback, unspecified, fc00::/7 (unique local), fe80::/10 (link-local), and IPv4-mapped addresses (checked against the IPv4 list).

### `validate_url`
Resolves the URL's hostname via DNS and checks every resolved IP against `is_private_ip`. Blocks the URL if any resolved address is private. Only permits HTTP and HTTPS schemes.

## Flash Processing (`flash.rs`)

Code adapted from the Ruffle Flash emulator (MIT licensed). Handles decoding various Flash image formats.

### `determine_jpeg_tag_format`
Identifies the image format inside a DefineBitsJPEG2/3 tag by checking magic bytes:
- JPEG: Starts with `0xFF 0xD8` or `0xFF 0xD9 0xFF 0xD8`.
- PNG: Starts with `0x89 0x50 0x4E 0x47`.
- GIF: Starts with `GIF8`.

### `decode_define_bits_jpeg`
Decodes JPEG/PNG/GIF data from DefineBitsJPEG2/3 tags. For true JPEG data with alpha (DefineBitsJPEG3), applies pre-multiplied alpha channel. Clamps color channels to alpha value to fix incorrect SWF encodings.

### `glue_tables_to_jpeg`
Combines JPEG encoding tables from a JpegTables tag with data from a DefineBits tag. Removes the trailing EOI marker from the tables and the leading SOI marker from the data before concatenation.

### `remove_invalid_jpeg_data`
Removes the invalid `0xFF 0xD9 0xFF 0xD8` (EOI+SOI) marker sequence from SWF JPEG data. This sequence can appear anywhere before the SOF marker in older SWF files. Also appends an EOI marker if missing.

### `decode_define_bits_lossless`
Decodes DefineBitsLossless tags (version 1 or 2):
- **RGB15 format**: 15-bit RGB, packed as 16-bit values. Unpacks to RGBA.
- **RGB32 format**: 32-bit ARGB, reordered to RGBA.
- **ColorMap8 format**: Palettized with up to 256 colors.

### `decompress_zlib`
Decompresses zlib-compressed data with a maximum output size of 128 MB.

## Hash (`hash.rs`)

### `PostHash`
Stores a post ID and its content hash (BLAKE3 keyed hash of the post ID). Used to construct file paths and URLs:
- `content_url(mime_type)`: Returns the URL to the post content file.
- `thumbnail_url()`: Returns the URL to the thumbnail (custom if it exists, generated otherwise).
- `content_path(mime_type)`: Returns the absolute filesystem path to the post content.
- `generated_thumbnail_path()`: Returns the path to the generated thumbnail.
- `custom_thumbnail_path()`: Returns the path to the custom thumbnail.

The Display trait formats the path as `{outer_bucket:06}/{inner_bucket:02}/{post_id}_{hash}`, where:
- `outer_bucket = post_id / 1_000_000`
- `inner_bucket = (post_id / 10_000) % 100`

### `GenericChecksum<N>`
A fixed-size checksum wrapper. Type aliases:
- `Checksum = GenericChecksum<32>` (BLAKE3).
- `Md5Checksum = GenericChecksum<16>` (MD5).

Implements `FromSql`/`ToSql` for PostgreSQL BYTEA type. Implements `FromStr` for hex string parsing.

### `gravatar_url`
Generates a Gravatar URL from a username. Uses BLAKE3 hash of the lowercase username formatted as a hex string.

### `compute_checksums`
Computes BLAKE3 and MD5 checksums of a file in a single pass using a 4 MB buffer. Reads the file in chunks, feeding each chunk to both hashers simultaneously.

### `compute_url_safe_hash`
Computes a base64 URL-safe (no padding) BLAKE3 hash of arbitrary bytes.

## Signature (`signature.rs`)

### Purpose
Computes a compact "signature" for an image that can be used for similarity search. Implementation follows the paper by H. Chi Wong, Marshall Bern and David Goldberg.

### Constants
- `GRID_SIZE = 9`: Size of the square grid used to compute the signature.
- `LUMINANCE_LEVELS = 2`: Two shades of light/dark plus identical, giving 5 possible symbols.
- `NUM_LETTERS = 12`: Length of each index word.
- `SIGNATURE_LEN = 548`: Total signature length (computed formula).
- `NUM_WORDS = 100`: Number of index words per signature.
- `COMPRESSED_SIGNATURE_LEN = 32`: Compressed length (ceil of SIGNATURE_LEN / SIGNATURE_DIGITS).

### `compute`
The main signature computation pipeline:
1. Convert image to grayscale.
2. Compute grid points with crop-based placement.
3. Compute average intensity matrix at each grid point.
4. Compute differences between neighboring grid squares.
5. Normalize differences into the range [0, 4].
6. Compress for database storage.

### `distance`
Computes a normalized distance (0 to 1) between two image signatures. Lower values indicate more similar images. Uses L2 distance divided by the sum of norms.

### `generate_indexes`
Creates 100 index words from a compressed signature. Each word is 12 letters, each letter is clamped to [-1, 1] (3 possible values). Words overlap across the signature. Each word is encoded as a base-3 number packed into an i32, with the word index stored in the high trits for fast PostgreSQL `&&` array overlap queries.

### Grid Point Computation
- `compute_row_deltas`: Sums absolute differences of neighboring row pixel intensities.
- `compute_column_deltas`: Sums absolute differences of neighboring column pixel intensities.
- `crop`: Determines crop bounds by analyzing where cumulative delta changes reach 1% of the total.
- `compute_grid_points`: Uses cropped bounds to place 9x9 grid points with radius sized at 5% of the smaller dimension.

### `compute_intensity_matrix`
For each grid point, computes the average pixel intensity within a square of radius `grid_square_radius`, clipped to image bounds.

### `compute_differences`
For each interior grid square (not on the boundary), computes the difference in average intensity with each of its 8 neighbors. Boundary squares are excluded from the final signature (8 * 7^2 + 20 * 7 + 12 = 548 differences).

### `normalize`
Maps each difference value to a symbol in [0, 4] based on cutoff thresholds. Cutoffs are computed so that dark differences, identical values, and light differences each receive roughly equal numbers of symbols.

### `compress` / `uncompress`
Compresses the signature by packing `SIGNATURE_DIGITS` base-`NUM_SYMBOLS` digits into each i64. Uncompressing reverses this by repeated division.

### `SignatureCache`
Caches an uncompressed signature and its L2 norm for efficient distance calculations.

## Thumbnail (`thumbnail.rs`)

### `ThumbnailType`
- `Post`: Thumbnail size defined by `config.thumbnails.post_dimensions()`.
- `Avatar`: Thumbnail size defined by `config.thumbnails.avatar_dimensions()`.

### `ThumbnailCategory`
- `Generated`: Auto-generated thumbnail.
- `Custom`: User-uploaded custom thumbnail.

### `create`
Creates a thumbnail:
1. Gets target dimensions from config based on thumbnail type.
2. Resizes the image so the smaller dimension is constrained, preserving aspect ratio. Uses Gaussian filtering.
3. If the image has an alpha channel, composites transparent pixels onto a solid white background to prevent JPEG corruption.

## Upload (`upload.rs`)

### `UploadToken`
A UUID-based token identifying a temporarily uploaded file. Serializes as a string, deserializes by splitting on the last period to extract the file extension for MIME type determination. Path is `{temporary_uploads_dir}/{uuid}.{extension}`.

### `PartName`
An enum for multipart form field names: `Content`, `Thumbnail`, `Avatar`.

### `Body<N>`
Result of multipart form extraction. Contains:
- `files`: An array of `N` optional UploadTokens, one per expected part.
- `metadata`: Optional raw bytes of a JSON metadata field.

### `extract`
Extracts fields from a multipart form:
1. Iterates over each field in the form data.
2. Matches field names against expected part names or "metadata".
3. For file parts, determines the MIME type from the filename extension or content type.
4. For the metadata part, validates that the content type is `application/json`.
5. Saves file parts to disk via `filesystem::save_uploaded_file`.

### `get_mime_type`
Determines MIME type from a multipart field by checking both the filename extension and the content type header. If both are present but disagree, returns a `ContentTypeMismatch` error.

### `verify_magic_bytes`
Validates that the first bytes of a file match the expected magic bytes for its declared MIME type:
- PNG: `89 50 4E 47`
- JPEG: `FF D8 FF`
- GIF: `GIF8`
- BMP: `BM`
- WebP: `RIFF....WEBP`
- MP4/MOV/AVIF: `....ftyp`
- WebM: `1A 45 DF A3`
- SWF: `FWS` or `CWS`
- PDF: `%PDF`
