# Content Decoding (`content/decode.rs`)

## Summary

This module decodes image and video content into pixel data. It supports raster images (BMP, GIF, JPEG, PNG, WebP, AVIF), vector formats (SWF/Flash), videos (MOV, MP4, WebM), and PDF documents. It uses the `image` crate for raster formats, `ffmpeg_sidecar` for video/AVIF, `swf` crate for Flash, and `pdftoppm` for PDF.

## Imports

- `api::error::{ApiError, ApiResult}`: Error types.
- `config::Config`: Application configuration.
- `content::{self, flash}`: Content module and Flash decoding utilities.
- `model::enums::{MimeType, PostType}`: MIME and post types.
- `ffmpeg_sidecar::command::FfmpegCommand`: FFmpeg CLI wrapper.
- `ffmpeg_sidecar::event::{FfmpegEvent, LogLevel}`: FFmpeg output events.
- `image::codecs::{gif::GifDecoder, webp::WebPDecoder}`: GIF and WebP decoders.
- `image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage, RgbaImage}`: Image decoding types.
- `std::fs::File`: File I/O.
- `std::io::BufReader`: Buffered reading.
- `std::path::Path`: File paths.
- `std::process::Command`: Process spawning (for pdftoppm).
- `swf::Tag`: SWF tag enum.
- `tracing::error`: Logging.

## Function: `representative_image(config, file_path, mime_type) -> ApiResult<DynamicImage>`

```rust
pub fn representative_image(config: &Config, file_path: &Path, mime_type: MimeType) -> ApiResult<DynamicImage>
```

Returns a representative image for the content at `file_path`. The method varies by MIME type:

| MIME Type | Method | Description |
|-----------|--------|-------------|
| `Bmp` | `image(path, ImageFormat::Bmp)` | Direct decode |
| `Gif` | `image(path, ImageFormat::Gif)` | Direct decode |
| `Jpeg` | `image(path, ImageFormat::Jpeg)` | Direct decode |
| `Png` | `image(path, ImageFormat::Png)` | Direct decode |
| `Webp` | `image(path, ImageFormat::WebP)` | Direct decode |
| `Swf` | `flash_image(config, path)` then `frame.ok_or(ApiError::EmptySwf)` | Largest SWF image; error if no decodable image found |
| `Avif` | `ffmpeg_frame(path, PostType::Image)` then `frame.ok_or(ApiError::FfmpegError(...))` | FFmpeg frame extraction; error if no frame |
| `Pdf` | `pdf_representative_image(file_path)` | First page rendered via pdftoppm |
| `Mov`, `Mp4`, `Webm` | `ffmpeg_frame(path, PostType::Video)` then `frame.ok_or(ApiError::EmptyVideo)` | FFmpeg thumbnail filter; error if no frame |

## Function: `video_has_audio(path) -> ApiResult<bool>`

Returns `true` if the video at `path` contains an audio stream.

**Algorithm:**
1. Convert path to string.
2. Spawn FFmpeg with arguments: `-c copy -t 0 -f null -` (copy codecs, 0 seconds, null output). This parses streams without decoding.
3. Iterate over FFmpeg events:
   - If `ParsedInputStream(stream)` where `stream.is_audio()`: return `Ok(true)`.
   - If `Log(Error|Fatal, err)` or `Event::Error(err)`: return `Err(ApiError::FfmpegError(err))`.
   - Ignore other events.
4. If loop completes without finding audio: return `Ok(false)`.

## Function: `swf_has_audio(path) -> ApiResult<bool>`

Returns `true` if the SWF at `path` contains audio tags.

**Algorithm:**
1. Open the file, wrap in `BufReader`.
2. Decompress SWF: `swf::decompress_swf(reader)`.
3. Parse SWF: `swf::parse_swf(&swf_buf)`.
4. Check tags for audio-related variants:
   - `DefineButtonSound`
   - `DefineSound`
   - `SoundStreamBlock`
   - `SoundStreamHead`
   - `SoundStreamHead2`
   - `StartSound`
   - `StartSound2`
5. Return `true` if any audio tag found, `false` otherwise.

## Function: `detect_post_type(file_path, mime_type) -> ApiResult<PostType>`

Returns the post type based on file content analysis.

**Algorithm:**
1. Define `image_type(animated: bool) -> PostType`: Returns `Animation` if true, `Image` otherwise.
2. Match on MIME type:

| MIME | Detection | Post Type |
|------|-----------|-----------|
| `Avif` | `avif_is_animated(path)` | Animation or Image |
| `Gif` | `gif_is_animated(path)` | Animation or Image |
| `Webp` | `webp_is_animated(path)` | Animation or Image |
| `Bmp`, `Jpeg`, `Png` | (static check) | Image |
| `Mp4`, `Mov`, `Webm` | (static check) | Video |
| `Swf` | (static check) | Flash |
| `Pdf` | (static check) | Document |

## Constant: `FFMPEG_PATH`

```rust
const FFMPEG_PATH: &str = "/opt/app/ffmpeg";
```

Path to the FFmpeg binary.

## Function: `image(file_path, format) -> ApiResult<DynamicImage>`

Decodes a raster image file.

**Algorithm:**
1. Open file with `File::open(file_path)`, wrap with `map_read_result`.
2. Create `ImageReader` with `BufReader`.
3. Set the format explicitly.
4. Set decoder limits to `image_reader_limits()`.
5. Call `reader.decode()`.

## Function: `ffmpeg_frame(path, post_type) -> ApiResult<Option<DynamicImage>>`

Extracts a representative frame using FFmpeg. Returns `None` if no frame is produced.

**Algorithm:**
1. Determine video format: `is_video_format = matches!(post_type, Video | Flash)`.
2. Choose filter:
   - Video/Flash: `"thumbnail,format=rgb24"` (thumbnail filter selects representative frame, RGB24 output).
   - Image (AVIF): `"format=rgba"` (RGBA for alpha support).
3. Build FFmpeg command:
   - Input: `path`
   - Arguments: `-t 30` (max 30 seconds read), `-vf {filter}`, `-frames:v 1` (single frame), `-f rawvideo` (raw pixel output), `-` (stdout).
4. Spawn and iterate events.
5. On `OutputFrame(f)`:
   - If video format: `RgbImage::from_raw(f.width, f.height, f.data)`, map to `DynamicImage::ImageRgb8`.
   - If image format: `RgbaImage::from_raw(f.width, f.height, f.data)`, map to `DynamicImage::ImageRgba8`.
   - If `from_raw` returns `None`: return `Err(ApiError::FrameBufferMismatch(width, height, buffer_len))`.
   - Otherwise: return `Ok(Some(extracted_frame))`.
6. On `Log(Error|Fatal, err)` or `Event::Error(err)`: return `Err(ApiError::FfmpegError(err))`.
7. Loop ends without frame: return `Ok(None)`.

## Function: `pdf_representative_image(file_path) -> ApiResult<DynamicImage>`

Renders the first page of a PDF as an image using `pdftoppm` (poppler-utils).

**Algorithm:**
1. Convert path to string.
2. Create output prefix: `format!("{path_str}.thumb")`.
3. Expected output file: `format!("{prefix}.png")` (pdftoppm appends `.png`).
4. Run: `pdftoppm -png -singlefile -f 1 -l 1 -r 100 {path_str} {prefix}`.
   - `-png`: Output PNG.
   - `-singlefile`: Single output file.
   - `-f 1 -l 1`: First page only.
   - `-r 100`: 100 DPI.
5. Check exit status:
   - `Ok(s) if s.success()`: Open the PNG file with `image::open`. Fall back to 1x1 black image on error.
   - Otherwise: Return 1x1 black image (`RgbImage::new(1, 1)`).
6. Remove the temporary PNG file (`let _ = std::fs::remove_file(&png_path)`).
7. Return result.

## Function: `flash_image(config, path) -> ApiResult<Option<DynamicImage>>`

Extracts the largest decodable image from a SWF file.

**Algorithm:**
1. Open and decompress SWF.
2. Parse SWF tags.
3. Search for `JpegTables` tag: stores JPEG encoding tables used by `DefineBits` tags. Find the first one.
4. Collect all images from image-bearing tags:
   - `DefineBits { jpeg_data }`: Glue JPEG tables to data via `flash::glue_tables_to_jpeg`, then decode as JPEG.
   - `DefineBitsLossless(bits)`: Call `flash::decode_define_bits_lossless(bits)`.
   - `DefineBitsJpeg2 { jpeg_data }`: Call `flash::decode_define_bits_jpeg(jpeg_data, None)`.
   - `DefineBitsJpeg3(bits)`: Call `flash::decode_define_bits_jpeg(bits.data, Some(bits.alpha_data))`.
   - Other tags: Skip.
   - Filter out errors (log and skip).
5. If `ffmpeg_frame(path, PostType::Flash)` returns a frame: push it to the images list. (Fallback for video-only SWFs.)
6. **Sort images by effective width after cropping:**
   - Get `(thumbnail_width, thumbnail_height)` from config.
   - For each image: compute effective width.
     - Condition `image.width() * thumbnail_height > thumbnail_width * image.height()`: aspect ratio wider than config, so crop to height. Effective width = `image.height() * thumbnail_width / thumbnail_height`.
     - Otherwise: effective width = `image.width()`.
   - Sort by `u32::MAX - effective_width` (descending).
7. Return the first (largest) image.

## Function: `avif_is_animated(path) -> ApiResult<bool>`

Checks if an AVIF file has multiple frames.

**Algorithm:**
1. Spawn FFmpeg to count video streams: `ffmpeg -i {path}`, filter for `ParsedInputStream(stream)` where `stream.is_video()`.
2. For each video stream index (0..count):
   - Spawn FFmpeg with: `-map 0:v:{stream_index} -frames:v 2 -vf "scale=1:1:flags=neighbor"` and `rawvideo()` output.
   - Count `OutputFrame` events.
   - If `frames > 1`: return `Ok(true)`.
3. If no stream had more than 1 frame: return `Ok(false)`.

Scaling to 1x1 pixels makes frame counting fast (nearest-neighbor scaling).

## Function: `gif_is_animated(path) -> ApiResult<bool>`

Checks if a GIF has multiple frames.

**Algorithm:**
1. Open file, create `GifDecoder` with `BufReader`, set limits.
2. GIF does not store frame count. Try to read the second frame: `decoder.into_frames().nth(1)`.
3. If `transpose()` returns `Some(frame)`: the GIF is animated (at least 2 frames).

## Function: `webp_is_animated(path) -> ApiResult<bool>`

Checks if a WebP has animation.

**Algorithm:**
1. Open file, create `WebPDecoder` with `BufReader`, set limits.
2. Call `decoder.has_animation()` (WebP stores animation flag in header).

## Function: `image_reader_limits() -> Limits`

```rust
fn image_reader_limits() -> Limits
```

Returns decoder limits with `max_alloc` set to 4 GB (4 * 1024^3 bytes). Other limits are unbounded (`Limits::no_limits()`).
