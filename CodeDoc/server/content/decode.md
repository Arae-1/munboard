# Content Decoding (`content/decode.rs`)

## Summary

This module decodes image and video content into pixel data. It supports raster images (BMP, GIF, JPEG, PNG, WebP, AVIF), videos (MOV, MP4, WebM), and PDF documents. It uses the `image` crate for raster formats, `ffmpeg_sidecar` for video/AVIF, and `pdftoppm` for PDF.

## Imports

- `api::error::{ApiError, ApiResult}`: Error types.
- `content`: Content module helpers (`map_read_result`).
- `model::enums::{MimeType, PostType}`: MIME and post types.
- `ffmpeg_sidecar::child::FfmpegChild`: FFmpeg child process handle.
- `ffmpeg_sidecar::command::FfmpegCommand`: FFmpeg CLI wrapper.
- `ffmpeg_sidecar::event::{FfmpegEvent, LogLevel}`: FFmpeg output events.
- `image::codecs::{gif::GifDecoder, webp::WebPDecoder}`: GIF and WebP decoders.
- `image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage, RgbaImage}`: Image decoding types.
- `std::fs::File`: File I/O.
- `std::io::BufReader`: Buffered reading.
- `std::path::Path`: File paths.
- `std::process::Command`: Process spawning (for pdftoppm).
- `std::sync::mpsc::{self, RecvTimeoutError}`: Event channel between ffmpeg parser threads and the caller.
- `std::time::{Duration, Instant}`: Timeouts and deadlines.

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_IMAGE_DIMENSION` | `16384` (16K) | Maximum width or height, in pixels, of an image or frame we are willing to decode. Rejects pathological dimensions before decoders allocate multi-GB buffers. |
| `MAX_DOCUMENT_RENDER_DIMENSION` | `4096` | Maximum width or height, in pixels, at which a PDF page may be rendered. PDF pages are only used for a thumbnail, so this smaller cap bounds a pathological render at about 0.7 s of CPU. Normal pages (letter at `-r 100` is about 850x1100) are unaffected. |
| `MAX_IMAGE_ALLOCATION` | `512 * 1024 * 1024` (512 MiB) | Maximum number of bytes image decoders may allocate while decoding. |
| `FFMPEG_TIMEOUT` | `Duration::from_secs(30)` | Wall-clock time limit for a single ffmpeg subprocess (probe or decode). |
| `PDFTOPP_TIMEOUT` | `Duration::from_secs(15)` | Wall-clock time limit for pdftoppm rendering. |

## Function: `representative_image(file_path, mime_type) -> ApiResult<DynamicImage>`

```rust
pub fn representative_image(file_path: &Path, mime_type: MimeType) -> ApiResult<DynamicImage>
```

Returns a representative image for the content at `file_path`. The method varies by MIME type:

| MIME Type | Method | Description |
|-----------|--------|-------------|
| `Bmp` | `image(path, ImageFormat::Bmp)` | Direct decode |
| `Gif` | `image(path, ImageFormat::Gif)` | Direct decode |
| `Jpeg` | `image(path, ImageFormat::Jpeg)` | Direct decode |
| `Png` | `image(path, ImageFormat::Png)` | Direct decode |
| `Webp` | `image(path, ImageFormat::WebP)` | Direct decode |
| `Avif` | `ffmpeg_frame(path, PostType::Image)` then `frame.ok_or(ApiError::FfmpegError("Unable to decode AVIF image with FFmpeg"))` | FFmpeg frame extraction; error if no frame |
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

## Function: `spawn_ffmpeg_events(path, args) -> ApiResult<(FfmpegChild, mpsc::Receiver<FfmpegEvent>)>`

Spawns ffmpeg with `args` and streams its parsed events through a channel.

**Algorithm:**
1. Spawn ffmpeg with the given input path and arguments.
2. Get the event iterator.
3. Create an `mpsc` channel and spawn a thread that forwards every event from the iterator into the channel. The thread stops when the receiver is dropped.
4. Return the child handle and the receiver. The caller uses the receiver with `recv_timeout` so the wait can be bounded by a deadline.

## Function: `recv_ffmpeg_event(child, rx, deadline) -> ApiResult<Option<FfmpegEvent>>`

Returns the next ffmpeg event before `deadline`, killing ffmpeg if the deadline is exceeded.

**Algorithm:**
1. Compute the remaining time: `deadline.saturating_duration_since(Instant::now())`.
2. Call `rx.recv_timeout(remaining)`:
   - `Ok(event)`: return `Ok(Some(event))`.
   - `RecvTimeoutError::Timeout`: kill the child, wait for it, and return `Err(ApiError::FfmpegError("FFmpeg exceeded its time limit"))`.
   - `RecvTimeoutError::Disconnected`: the process ended; return `Ok(None)`.

## Function: `probe_video_dimensions(path) -> ApiResult<Option<(u32, u32)>>`

Returns the coded dimensions of the first video stream without decoding any frames. Mirrors the stream probe used by `video_has_audio`.

**Algorithm:**
1. Spawn ffmpeg with `-c copy -t 0 -f null -` (copy codecs, 0 seconds, null output).
2. Enforce the `FFMPEG_TIMEOUT` deadline with `recv_ffmpeg_event`.
3. On `ParsedInputStream(stream)` with video data: kill the child and return `Ok(Some((width, height)))`.
4. On an error or fatal log: return `Err(ApiError::FfmpegError)`.
5. On process end without video data: return `Ok(None)`.

## Function: `ffmpeg_frame(path, post_type) -> ApiResult<Option<DynamicImage>>`

Extracts a representative frame using FFmpeg. Returns `None` if no frame is produced.

**Algorithm:**
1. Probe the coded dimensions with `probe_video_dimensions(path)`.
   - If either dimension exceeds `MAX_IMAGE_DIMENSION`: return `Err(ApiError::FfmpegError("Media dimensions exceed the maximum supported size"))`. This rejects oversized media before the decode pass allocates full-resolution frame buffers inside ffmpeg.
2. Determine video format: `is_video_format = matches!(post_type, PostType::Video)`.
3. Build the scale filter: `scale=w='min(16384,iw)':h='min(16384,ih)':force_original_aspect_ratio=decrease`. This scales frames down before their raw bytes are transferred into the process, so an oversized video cannot force a multi-GB frame allocation.
4. Choose filter:
   - Video: `"thumbnail,format=rgb24,{scale}"` (thumbnail filter selects representative frame, RGB24 output).
   - Image (AVIF): `"format=rgba,{scale}"` (RGBA for alpha support).
5. Spawn ffmpeg with arguments: `-threads 1 -t 30 -vf {filter} -frames:v 1 -f rawvideo -`.
   - `-threads 1` bounds decode CPU usage; the `FFMPEG_TIMEOUT` deadline bounds wall time.
6. Loop over events with the `FFMPEG_TIMEOUT` deadline:
   - On `OutputFrame(f)`:
     - If `f.width` or `f.height` exceeds `MAX_IMAGE_DIMENSION`: return `Err(ApiError::FfmpegError("Decoded frame exceeds maximum dimensions"))`.
     - If video format: `RgbImage::from_raw(f.width, f.height, f.data)`, map to `DynamicImage::ImageRgb8`.
     - If image format: `RgbaImage::from_raw(f.width, f.height, f.data)`, map to `DynamicImage::ImageRgba8`.
     - If `from_raw` returns `None`: return `Err(ApiError::FrameBufferMismatch(width, height, buffer_len))`.
     - Kill the child and wait for it (the frame is fully received; do not let ffmpeg run to completion in the background).
     - Return `Ok(Some(extracted_frame))`.
   - On `Log(Error|Fatal, err)` or `Event::Error(err)`: return `Err(ApiError::FfmpegError(err))`.
   - Other events: continue.
   - `Ok(None)` (process ended): break.
7. Loop ends without frame: return `Ok(None)`.

## Function: `pdf_representative_image(file_path) -> ApiResult<DynamicImage>`

Renders the first page of a PDF as an image using `pdftoppm` (poppler-utils). Falls back to a neutral 1x1 placeholder on any error so uploads never fail.

**Algorithm:**
1. Convert path to string.
2. Create output prefix: `format!("{path_str}.thumb")`.
3. Expected output file: `format!("{prefix}.png")` (pdftoppm appends `.png`).
4. Spawn: `pdftoppm -png -singlefile -f 1 -l 1 -r 100 -W 4096 -H 4096 {path_str} {prefix}`.
   - `-png`: Output PNG.
   - `-singlefile`: Single output file.
   - `-f 1 -l 1`: First page only.
   - `-r 100`: 100 DPI.
   - `-W 4096 -H 4096`: Cap the output size at `MAX_DOCUMENT_RENDER_DIMENSION` (4096 px) in each dimension. A pathological page (for example a crafted 100000x100000 pt page) cannot burn CPU rendering a full 16K image that the decode budget would reject anyway.
5. Enforce the `PDFTOPP_TIMEOUT` deadline (15 s): poll `child.try_wait()` every 25 ms. If the deadline passes, kill the child and treat the render as failed. `pdftoppm` has no built-in timeout, so the deadline is enforced here.
6. Check exit status:
   - `Ok(s) if s.success()`: decode the PNG with `decode_pdf_render(png_path)`. Fall back to 1x1 black image on error.
   - Otherwise: Return 1x1 black image (`RgbImage::new(1, 1)`).
7. Remove the temporary PNG file (`let _ = std::fs::remove_file(&png_path)`).
8. Return result.

## Function: `decode_pdf_render(png_path) -> ApiResult<DynamicImage>`

Decodes the PNG rendered by `pdftoppm` with the same limits as regular image uploads, so an oversized PDF page cannot force a multi-GB allocation.

**Algorithm:**
1. Open the PNG file with `map_read_result`.
2. Create an `ImageReader` with `BufReader`, set the format to `Png`.
3. Set decoder limits to `image_reader_limits()`.
4. Call `reader.decode()`.

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

Returns decoder limits applied to all image decodes in this module:
- `max_image_width = Some(MAX_IMAGE_DIMENSION)` (16384).
- `max_image_height = Some(MAX_IMAGE_DIMENSION)` (16384).
- `max_alloc = Some(MAX_IMAGE_ALLOCATION)` (512 MiB).

All other limits are unbounded (`Limits::no_limits()`).
