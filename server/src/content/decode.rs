use crate::api::error::{ApiError, ApiResult, ByteSize};
use crate::config::Config;
use crate::content::{self, flash};
use crate::model::enums::{MimeType, PostType};
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::{FfmpegEvent, LogLevel};
use image::codecs::{gif::GifDecoder, webp::WebPDecoder};
use image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits, RgbImage, RgbaImage};
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
use swf::Tag;
use tracing::error;

/// Maximum width or height (in pixels) of an image or frame we are willing to
/// decode. Rejects pathological dimensions before decoders allocate multi-GB
/// buffers.
const MAX_IMAGE_DIMENSION: u32 = 16384; // 16K

/// Maximum width or height (in pixels) at which a PDF page may be rendered.
/// PDF pages are only used to make a thumbnail, so this smaller cap bounds a
/// pathological render at ~0.7 s of CPU and stays well under the image decode
/// budget, while normal pages (letter at -r 100 ≈ 850x1100) are unaffected.
const MAX_DOCUMENT_RENDER_DIMENSION: u32 = 4096;

/// Maximum number of bytes image decoders may allocate while decoding.
const MAX_IMAGE_ALLOCATION: u64 = 512 * 1024 * 1024; // 512 MiB

/// Maximum declared uncompressed size of an SWF file. The SWF library
/// pre-allocates the declared size before parsing, so this prevents multi-GB
/// allocations from a crafted header (decompression bomb).
const MAX_SWF_UNCOMPRESSED_SIZE: u64 = 256 * 1024 * 1024; // 256 MiB

/// Returns a representative image for the given content.
/// For images, this is simply the decoded image.
/// For videos, `FFmpeg` determines the thumbnail.
/// For Flash media, it is the largest image that can be decoded from the Flash tags.
pub fn representative_image(config: &Config, file_path: &Path, mime_type: MimeType) -> ApiResult<DynamicImage> {
    match mime_type {
        MimeType::Bmp => image(file_path, ImageFormat::Bmp),
        MimeType::Gif => image(file_path, ImageFormat::Gif),
        MimeType::Jpeg => image(file_path, ImageFormat::Jpeg),
        MimeType::Png => image(file_path, ImageFormat::Png),
        MimeType::Webp => image(file_path, ImageFormat::WebP),
        MimeType::Swf => flash_image(config, file_path).and_then(|frame| frame.ok_or(ApiError::EmptySwf)),
        MimeType::Avif => ffmpeg_frame(file_path, PostType::Image)
            .and_then(|frame| frame.ok_or(ApiError::FfmpegError("Unable to decode AVIF image with FFmpeg".into()))),
        MimeType::Pdf => pdf_representative_image(file_path),
        MimeType::Mov | MimeType::Mp4 | MimeType::Webm => {
            ffmpeg_frame(file_path, PostType::Video).and_then(|frame| frame.ok_or(ApiError::EmptyVideo))
        }
    }
}

/// Returns if the video at `path` has an audio channel.
pub fn video_has_audio(path: &Path) -> ApiResult<bool> {
    let path_str = path.to_string_lossy();
    let iter = FfmpegCommand::new_with_path(FFMPEG_PATH)
        .input(path_str)
        .args(["-c", "copy", "-t", "0", "-f", "null", "-"])
        .spawn()?
        .iter()
        .map_err(|err| ApiError::FfmpegError(err.into_boxed_dyn_error()))?;

    for event in iter {
        match event {
            FfmpegEvent::ParsedInputStream(stream) if stream.is_audio() => {
                return Ok(true);
            }
            FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) | FfmpegEvent::Error(err) => {
                return Err(ApiError::FfmpegError(err.into()));
            }
            _ => {}
        }
    }
    Ok(false)
}

/// Decompresses the SWF file at `path`, after rejecting files whose declared
/// uncompressed length exceeds [`MAX_SWF_UNCOMPRESSED_SIZE`].
///
/// The SWF library allocates a buffer of the declared size up front, so a
/// crafted header could otherwise force a multi-GB pre-allocation.
fn decompress_swf_file(path: &Path) -> ApiResult<swf::SwfBuf> {
    let mut reader = BufReader::new(File::open(path)?);
    // SWF header: 3-byte signature ("FWS"/"CWS"/"ZWS"), 1-byte version, then a
    // little-endian u32 giving the uncompressed file length (header included).
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;
    if !header.starts_with(b"FWS") && !header.starts_with(b"CWS") && !header.starts_with(b"ZWS") {
        return Err(swf::error::Error::invalid_data("Invalid SWF signature").into());
    }
    let uncompressed_len = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    if u64::from(uncompressed_len) > MAX_SWF_UNCOMPRESSED_SIZE {
        return Err(ApiError::FileTooLarge(ByteSize(MAX_SWF_UNCOMPRESSED_SIZE as usize)));
    }
    reader.seek(std::io::SeekFrom::Start(0))?;
    let swf_buf = swf::decompress_swf(reader)?;
    // A crafted zlib stream can inflate beyond its declared length, so also
    // reject the result if it actually exceeds the budget.
    if swf_buf.data.len() as u64 > MAX_SWF_UNCOMPRESSED_SIZE {
        return Err(ApiError::FileTooLarge(ByteSize(MAX_SWF_UNCOMPRESSED_SIZE as usize)));
    }
    Ok(swf_buf)
}

/// Returns if the swf at `path` has audio.
pub fn swf_has_audio(path: &Path) -> ApiResult<bool> {
    let swf_buf = decompress_swf_file(path)?;
    let swf = swf::parse_swf(&swf_buf)?;

    Ok(swf.tags.iter().any(|tag| {
        matches!(
            tag,
            Tag::DefineButtonSound(_)
                | Tag::DefineSound(_)
                | Tag::SoundStreamBlock(_)
                | Tag::SoundStreamHead(_)
                | Tag::SoundStreamHead2(_)
                | Tag::StartSound(_)
                | Tag::StartSound2 { .. }
        )
    }))
}

/// Returns the post type based on file content.
/// For image formats that support animation, it checks the file content for multiple frames.
/// For everything else, it just checks the mime type.
pub fn detect_post_type(file_path: &Path, mime_type: MimeType) -> ApiResult<PostType> {
    // Shorthand to return PostType::Animation or PostType::Image based on bool input
    let image_type = |animated: bool| -> PostType { if animated { PostType::Animation } else { PostType::Image } };
    match mime_type {
        MimeType::Avif => avif_is_animated(file_path).map(image_type),
        MimeType::Gif => gif_is_animated(file_path).map(image_type),
        MimeType::Webp => webp_is_animated(file_path).map(image_type),
        MimeType::Bmp | MimeType::Jpeg | MimeType::Png => Ok(PostType::Image),
        MimeType::Mp4 | MimeType::Mov | MimeType::Webm => Ok(PostType::Video),
        MimeType::Swf => Ok(PostType::Flash),
        MimeType::Pdf => Ok(PostType::Document),
    }
}

const FFMPEG_PATH: &str = "/opt/app/ffmpeg";

/// Wall-clock time limit for a single ffmpeg subprocess (probe or decode).
const FFMPEG_TIMEOUT: Duration = Duration::from_secs(30);
/// Wall-clock time limit for pdftoppm rendering.
const PDFTOPP_TIMEOUT: Duration = Duration::from_secs(15);

/// Decodes a raw array of bytes into pixel data.
fn image(file_path: &Path, format: ImageFormat) -> ApiResult<DynamicImage> {
    let file = content::map_read_result(File::open(file_path))?;

    let mut reader = ImageReader::new(BufReader::new(file));
    reader.set_format(format);
    reader.limits(image_reader_limits());
    reader.decode().map_err(ApiError::from)
}

/// Spawns ffmpeg with `args` and streams its parsed events through a channel.
/// Returns the child handle alongside the receiver so the caller can enforce a
/// deadline and kill the process on timeout.
fn spawn_ffmpeg_events(path: &Path, args: &[&str]) -> ApiResult<(FfmpegChild, mpsc::Receiver<FfmpegEvent>)> {
    let path_str = path.to_string_lossy();
    let mut child = FfmpegCommand::new_with_path(FFMPEG_PATH)
        .input(&path_str)
        .args(args)
        .spawn()?;
    let iter = child.iter().map_err(|err| ApiError::FfmpegError(err.into_boxed_dyn_error()))?;
    let (tx, rx) = mpsc::channel();
    // Forward events from the parser threads so the wait can be bounded with
    // `recv_timeout` instead of blocking indefinitely.
    std::thread::spawn(move || {
        for event in iter {
            if tx.send(event).is_err() {
                break;
            }
        }
    });
    Ok((child, rx))
}

/// Returns the next ffmpeg event before `deadline`, killing ffmpeg if it is
/// exceeded. Returns `Ok(None)` once the process has ended.
fn recv_ffmpeg_event(
    child: &mut FfmpegChild,
    rx: &mpsc::Receiver<FfmpegEvent>,
    deadline: Instant,
) -> ApiResult<Option<FfmpegEvent>> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    match rx.recv_timeout(remaining) {
        Ok(event) => Ok(Some(event)),
        Err(RecvTimeoutError::Timeout) => {
            let _ = child.as_inner_mut().kill();
            let _ = child.wait();
            Err(ApiError::FfmpegError("FFmpeg exceeded its time limit".into()))
        }
        Err(RecvTimeoutError::Disconnected) => Ok(None),
    }
}

/// Returns the coded dimensions of the first video stream without decoding any
/// frames. Mirrors the stream probe used by [`video_has_audio`].
fn probe_video_dimensions(path: &Path) -> ApiResult<Option<(u32, u32)>> {
    let (mut child, rx) = spawn_ffmpeg_events(path, &["-c", "copy", "-t", "0", "-f", "null", "-"])?;
    let deadline = Instant::now() + FFMPEG_TIMEOUT;
    loop {
        match recv_ffmpeg_event(&mut child, &rx, deadline)? {
            Some(FfmpegEvent::ParsedInputStream(stream)) => {
                if let Some(video) = stream.video_data() {
                    let _ = child.as_inner_mut().kill();
                    let _ = child.wait();
                    return Ok(Some((video.width, video.height)));
                }
            }
            Some(FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) | FfmpegEvent::Error(err)) => {
                return Err(ApiError::FfmpegError(err.into()));
            }
            Some(_) => {}
            None => return Ok(None),
        }
    }
}

/// Decodes a representative frame of the image or video at the given `path`.
fn ffmpeg_frame(path: &Path, post_type: PostType) -> ApiResult<Option<DynamicImage>> {
    // Reject media whose coded dimensions exceed the limit before the decode
    // pass allocates full-resolution frame buffers inside ffmpeg.
    if let Some((width, height)) = probe_video_dimensions(path)? {
        if width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION {
            return Err(ApiError::FfmpegError("Media dimensions exceed the maximum supported size".into()));
        }
    }

    let is_video_format = matches!(post_type, PostType::Video | PostType::Flash);
    // Scale frames down to the decode dimension limit before their raw bytes
    // are transferred into this process, so an oversized video cannot force a
    // multi-GB frame allocation.
    let scale = format!("scale=w='min({MAX_IMAGE_DIMENSION},iw)':h='min({MAX_IMAGE_DIMENSION},ih)':force_original_aspect_ratio=decrease");
    let filter = if is_video_format {
        format!("thumbnail,format=rgb24,{scale}")
    } else {
        format!("format=rgba,{scale}")
    };

    // `-threads 1` bounds decode CPU usage; the deadline below bounds wall time.
    let (mut child, rx) = spawn_ffmpeg_events(
        path,
        &["-threads", "1", "-t", "30", "-vf", filter.as_str(), "-frames:v", "1", "-f", "rawvideo", "-"],
    )?;
    let deadline = Instant::now() + FFMPEG_TIMEOUT;
    loop {
        match recv_ffmpeg_event(&mut child, &rx, deadline)? {
            Some(FfmpegEvent::OutputFrame(f)) => {
                if f.width > MAX_IMAGE_DIMENSION || f.height > MAX_IMAGE_DIMENSION {
                    return Err(ApiError::FfmpegError("Decoded frame exceeds maximum dimensions".into()));
                }
                let buffer_len = f.data.len();
                let extracted_frame = if is_video_format {
                    RgbImage::from_raw(f.width, f.height, f.data).map(DynamicImage::ImageRgb8)
                } else {
                    RgbaImage::from_raw(f.width, f.height, f.data).map(DynamicImage::ImageRgba8)
                }
                .ok_or(ApiError::FrameBufferMismatch(f.width, f.height, buffer_len))?;
                // The frame is fully received; stop ffmpeg instead of letting
                // it run to completion in the background.
                let _ = child.as_inner_mut().kill();
                let _ = child.wait();
                return Ok(Some(extracted_frame));
            }
            Some(FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) | FfmpegEvent::Error(err)) => {
                return Err(ApiError::FfmpegError(err.into()));
            }
            Some(_) => {}
            None => break,
        }
    }
    Ok(None)
}

/// Renders the first page of a PDF as a PNG via pdftoppm (poppler).
/// Falls back to a neutral 1x1 placeholder on any error so uploads never fail.
fn pdf_representative_image(file_path: &Path) -> ApiResult<DynamicImage> {
    let path_str = file_path.to_string_lossy();
    // pdftoppm appends ".png" to the given prefix; render next to the source file.
    let prefix = format!("{}.thumb", path_str);
    let png_path = format!("{prefix}.png");

    let max_dimension = MAX_DOCUMENT_RENDER_DIMENSION.to_string();
    // `-W`/`-H` cap the output size so a pathological page (e.g. a crafted
    // 100000x100000 pt page) cannot burn CPU rendering a full 16K image that
    // the decode budget would reject anyway. pdftoppm has no built-in
    // timeout, so enforce one here and kill the process if it is exceeded.
    let child = Command::new("pdftoppm")
        .args([
            "-png", "-singlefile", "-f", "1", "-l", "1", "-r", "100",
            "-W", &max_dimension, "-H", &max_dimension, &path_str, &prefix,
        ])
        .spawn();

    let result = match child {
        Ok(mut child) => {
            let deadline = Instant::now() + PDFTOPP_TIMEOUT;
            let status = loop {
                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) => {}
                    Err(_) => break Err(()),
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(());
                }
                std::thread::sleep(Duration::from_millis(25));
            };
            match status {
                Ok(s) if s.success() => decode_pdf_render(Path::new(&png_path))
                    .or_else(|_| Ok(DynamicImage::ImageRgb8(RgbImage::new(1, 1)))),
                _ => Ok(DynamicImage::ImageRgb8(RgbImage::new(1, 1))),
            }
        }
        Err(_) => Ok(DynamicImage::ImageRgb8(RgbImage::new(1, 1))),
    };
    let _ = std::fs::remove_file(&png_path);
    result
}

/// Decodes the PNG rendered by pdftoppm with the same limits as regular image
/// uploads, so an oversized PDF page cannot force a multi-GB allocation.
fn decode_pdf_render(png_path: &Path) -> ApiResult<DynamicImage> {
    let file = content::map_read_result(File::open(png_path))?;
    let mut reader = ImageReader::new(BufReader::new(file));
    reader.set_format(ImageFormat::Png);
    reader.limits(image_reader_limits());
    reader.decode().map_err(ApiError::from)
}

/// Search swf tags for the largest decodable image
fn flash_image(config: &Config, path: &Path) -> ApiResult<Option<DynamicImage>> {
    let swf_buf = decompress_swf_file(path)?;
    let swf = swf::parse_swf(&swf_buf)?;

    let encoding_table = swf
        .tags
        .iter()
        .find_map(|tag| {
            if let Tag::JpegTables(table) = tag {
                Some(table)
            } else {
                None
            }
        })
        .copied();
    let mut images: Vec<_> = swf
        .tags
        .iter()
        .filter_map(|tag| match tag {
            Tag::DefineBits { id: _, jpeg_data } => {
                let jpeg_data = flash::glue_tables_to_jpeg(jpeg_data, encoding_table);
                Some(flash::decode_embedded_image(&jpeg_data, ImageFormat::Jpeg))
            }
            Tag::DefineBitsLossless(bits) => flash::decode_define_bits_lossless(bits).transpose(),
            Tag::DefineBitsJpeg2 { id: _, jpeg_data } => Some(flash::decode_define_bits_jpeg(jpeg_data, None)),
            Tag::DefineBitsJpeg3(bits) => Some(flash::decode_define_bits_jpeg(bits.data, Some(bits.alpha_data))),
            _ => None,
        })
        .filter_map(|image_result| match image_result {
            Ok(image) => Some(image),
            Err(err) => {
                error!("Failure to decode flash image for reason: {err}");
                None
            }
        })
        .collect();

    // Some Flash files only have video frames, which are hard to decode.
    // So, we feed to ffmpeg and see if it can decode a representaive frame.
    if let Ok(Some(frame)) = ffmpeg_frame(path, PostType::Flash) {
        images.push(frame);
    }

    // Sort images in order of decreasing effective width after cropping for thumbnails
    images.sort_by_key(|image| {
        let (thumbnail_width, thumbnail_height) = config.thumbnails.post_dimensions();

        // Condition is equivalent to image_aspect_ratio > config_thumbnail_aspect_ratio
        let effective_width = if image.width() * thumbnail_height > thumbnail_width * image.height() {
            image.height() * thumbnail_width / thumbnail_height
        } else {
            image.width()
        };
        u32::MAX - effective_width
    });
    Ok(images.into_iter().next())
}

/// Uses `FFmpeg` to determine if a file contains multiple frames
fn avif_is_animated(path: &Path) -> ApiResult<bool> {
    let path_str = path.to_string_lossy();
    let video_stream_count = FfmpegCommand::new_with_path(FFMPEG_PATH)
        .input(&path_str)
        .spawn()?
        .iter()
        .map_err(|err| ApiError::FfmpegError(err.into_boxed_dyn_error()))?
        .filter(|event| matches!(event, FfmpegEvent::ParsedInputStream(stream) if stream.is_video()))
        .count();

    for stream_index in 0..video_stream_count {
        let iter = FfmpegCommand::new_with_path(FFMPEG_PATH)
            .input(&path_str)
            .args([
                "-map",
                &format!("0:v:{stream_index}"),
                "-frames:v",
                "2",
                "-vf",
                "scale=1:1:flags=neighbor",
            ])
            .rawvideo()
            .spawn()?
            .iter()
            .map_err(|err| ApiError::FfmpegError(err.into_boxed_dyn_error()))?;

        let mut frames = 0;
        for event in iter {
            match event {
                FfmpegEvent::OutputFrame(_) => frames += 1,
                FfmpegEvent::Log(LogLevel::Error | LogLevel::Fatal, err) | FfmpegEvent::Error(err) => {
                    return Err(ApiError::FfmpegError(err.into()));
                }
                _ => {}
            }
        }
        if frames > 1 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn gif_is_animated(path: &Path) -> ApiResult<bool> {
    let file = content::map_read_result(File::open(path))?;
    let mut decoder = GifDecoder::new(BufReader::new(file))?;
    decoder.set_limits(image_reader_limits())?;

    // GIF doesn't store a frame count, so just check for a second frame.
    let mut frames = decoder.into_frames();
    frames
        .nth(1)
        .transpose()
        .map(|frame| frame.is_some())
        .map_err(ApiError::from)
}

fn webp_is_animated(path: &Path) -> ApiResult<bool> {
    let file = content::map_read_result(File::open(path))?;
    let mut decoder = WebPDecoder::new(BufReader::new(file))?;
    decoder.set_limits(image_reader_limits())?;
    Ok(decoder.has_animation())
}

/// Returns maximum decoded image size.
fn image_reader_limits() -> Limits {
    let mut limits = Limits::no_limits();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_IMAGE_ALLOCATION);
    limits
}
