use crate::api::error::{ApiError, ApiResult};
use crate::config::Config;
use crate::filesystem::{self, Directory};
use crate::model::enums::MimeType;
use crate::string::SmallString;
use axum::extract::multipart::{Field, Multipart};
use axum::extract::rejection::{JsonRejection, MissingJsonContentType};
use serde::{Deserialize, Deserializer, Serialize};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use strum::IntoStaticStr;
use uuid::Uuid;

const MAX_METADATA_SIZE: usize = 1024_usize.pow(2);

/// A token that represents a file that's been streamed to disk during upload.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct UploadToken {
    token: String,
    #[serde(skip)]
    mime_type: MimeType,
}

impl UploadToken {
    pub fn new(mime_type: MimeType) -> Self {
        let token = format!("{}.{}", Uuid::new_v4(), mime_type.extension());
        Self { token, mime_type }
    }

    pub fn mime_type(&self) -> MimeType {
        self.mime_type
    }

    pub fn path(&self, config: &Config) -> PathBuf {
        config.path(Directory::TemporaryUploads).join(&self.token)
    }
}

impl<'de> Deserialize<'de> for UploadToken {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let token = String::deserialize(deserializer)?;
        if token.contains('/') || token.contains('\\') {
            return Err(serde::de::Error::custom("invalid upload token"));
        }

        let (_uuid, extension) = token.split_once('.').unwrap_or((&token, ""));
        let mime_type = MimeType::from_extension(extension).map_err(serde::de::Error::custom)?;
        Ok(Self { token, mime_type })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum PartName {
    Content,
    Thumbnail,
    Avatar,
}

pub struct Body<const N: usize> {
    pub files: [Option<UploadToken>; N],
    pub metadata: Option<Vec<u8>>,
}

/// Attempts to extract given `fields` and optional JSON "metadata" field from given `form_data`.
pub async fn extract<const N: usize>(
    config: &Config,
    mut form_data: Multipart,
    fields: [PartName; N],
) -> ApiResult<Body<N>> {
    let mut files = std::array::from_fn(|_| None);
    let mut metadata = None;
    while let Some(mut field) = form_data.next_field().await? {
        let position = fields
            .iter()
            .map(Into::<&str>::into)
            .position(|name| field.name() == Some(name));
        if position.is_none() && field.name() != Some("metadata") {
            continue;
        }

        // Get MIME type from field
        let file_info = position
            .map(|index| get_mime_type(&field).map(|mime_type| (index, mime_type)))
            .transpose()?;

        // Ensure metadata is JSON
        if file_info.is_none() && field.content_type() != Some("application/json") {
            return Err(ApiError::JsonRejection(JsonRejection::MissingJsonContentType(
                MissingJsonContentType::default(),
            )));
        }

        match file_info {
            Some((index, mime_type)) => {
                files[index] = filesystem::save_uploaded_file(config, field, mime_type, Some(config.max_upload_size))
                    .await
                    .map(Some)?;
            }
            None => {
                let mut total_bytes = 0usize;
                let mut bytes = Vec::new();
                while let Some(chunk) = field.chunk().await? {
                    total_bytes += chunk.len();
                    if total_bytes > MAX_METADATA_SIZE {
                        return Err(ApiError::MetadataTooLarge(MAX_METADATA_SIZE));
                    }
                    bytes.extend_from_slice(&chunk);
                }
                metadata = Some(bytes);
            }
        }
    }
    Ok(Body { files, metadata })
}

/// Returns the MIME type of the given part.
/// It either gets this from the filename extension or the content type if no extension exists.
/// If both exist but their content types are different, an error is returned.
fn get_mime_type(field: &Field) -> ApiResult<MimeType> {
    let extension = field
        .file_name()
        .map(Path::new)
        .and_then(Path::extension)
        .and_then(OsStr::to_str);
    let content_type = field.content_type().map(str::trim);

    match (extension, content_type) {
        (Some(ext), None | Some("application/octet-stream")) => MimeType::from_extension(ext).map_err(ApiError::from),
        (Some(ext), Some(content_type)) => {
            let mime_type = MimeType::from_extension(ext)?;
            if MimeType::from_str(content_type) != Ok(mime_type) {
                return Err(ApiError::ContentTypeMismatch(mime_type, SmallString::new(content_type)));
            }
            Ok(mime_type)
        }
        (None, Some(content_type)) => MimeType::from_str(content_type)
            .map_err(Box::from)
            .map_err(ApiError::from),
        (None, None) => Err(ApiError::MissingContentType),
    }
}

/// Verifies that the magic bytes at the start of the file at `path` match the claimed `mime_type`.
pub fn verify_magic_bytes(path: &Path, mime_type: MimeType) -> ApiResult<()> {
    let mut file = std::fs::File::open(path)?;
    let mut bytes = [0u8; 12];
    let bytes_read = file.read(&mut bytes)?;
    let magic = &bytes[..bytes_read];

    let valid = match mime_type {
        MimeType::Png => magic.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        MimeType::Jpeg => magic.starts_with(&[0xFF, 0xD8, 0xFF]),
        MimeType::Gif => magic.starts_with(b"GIF8"),
        MimeType::Bmp => magic.starts_with(b"BM"),
        MimeType::Webp => magic.starts_with(b"RIFF") && magic.len() >= 12 && &magic[8..12] == b"WEBP",
        MimeType::Mp4 | MimeType::Mov | MimeType::Avif => magic.len() >= 8 && &magic[4..8] == b"ftyp",
        MimeType::Webm => magic.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]),
        MimeType::Swf => magic.starts_with(b"FWS") || magic.starts_with(b"CWS"),
        MimeType::Pdf => magic.starts_with(b"%PDF"),
    };

    if valid {
        Ok(())
    } else {
        Err(ApiError::InvalidMagicBytes(mime_type))
    }
}
