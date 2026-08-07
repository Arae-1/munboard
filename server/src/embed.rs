//! Serves HTML pages with OpenGraph/Twitter meta tags for post URLs.
//!
//! Chat applications (Discord, Telegram) fetch shared links server-side to
//! render rich embeds. They do not run JavaScript, so the SPA client cannot
//! provide the embed. This module serves a small HTML page for each post so
//! crawlers can render a preview. The page also boots the SPA in browsers.

use crate::app::AppState;
use crate::auth::Client as AuthClient;
use crate::config::{Config, EmbedImage};
use crate::content::hash::PostHash;
use crate::model::enums::{MimeType, PostSafety, PostType, UserRank};
use crate::model::tag::TagName;
use crate::schema::{post, post_statistics, post_tag, tag, tag_category, tag_name, user};
use crate::search::preferences;
use crate::string::{LargeString, SmallString};
use crate::time::DateTime;
use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::OptionalExtension;
use html_escape::encode_text;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use tracing::error;

/// HTML shell template with `%%TITLE%%`, `%%OG_TAGS%%`, `%%BASE_HREF%%`, and `%%ICON_LINK%%` placeholders.
const PAGE_TEMPLATE: &str = include_str!("embed/page.html");

/// Matches the original URI of the requested page, capturing any subdirectory
/// prefix so relative asset links keep working when the SPA is served from a
/// subpath (e.g. `/booru/post/123`).
static ORIGINAL_URI_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(.+?)/post/[0-9]+/?$").unwrap());

/// Returns a router serving `OpenGraph` embed pages. This is a plain axum router,
/// not part of the `OpenAPI` JSON API router, because it responds with HTML.
pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/embed/post/{id}", get(post_page))
        .with_state(state)
}

/// Row fetched from the `post` table for embed rendering.
type PostEmbedRow = (
    i64,
    Option<i64>,
    PostSafety,
    PostType,
    MimeType,
    i32,
    i32,
    i64,
    DateTime,
    Option<DateTime>,
    LargeString,
);

/// Maximum number of tag names rendered in a single `{tags}` placeholder slot.
const MAX_TAGS_PER_SLOT: usize = 10;

async fn post_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
) -> Response {
    let site_name: &str = &state.config.public_info.name;

    // Serve a plain page when embeds are disabled so browsers still work.
    if !state.config.enable_embeds {
        return html_page(build_page(site_name, "", "/", &icon_link("", "")));
    }

    let host = headers.get("host").and_then(|value| value.to_str().ok());
    let site_base = site_base_url(&state.config, host, &headers);
    let icon_link = icon_link(&state.config.embed.logo_url, &site_base);

    let mut conn = match state.connection_pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            error!(?err, "Failed to get database connection");
            return server_error();
        }
    };

    let (id, user_id, safety, post_type, mime_type, width, height, file_size, creation_time, launch_date, source): PostEmbedRow =
        match post::table
            .find(post_id)
            .select((
                post::id,
                post::user_id,
                post::safety,
                post::type_,
                post::mime_type,
                post::width,
                post::height,
                post::file_size,
                post::creation_time,
                post::launch_date,
                post::source,
            ))
            .first(conn.as_mut())
            .optional()
        {
            Ok(Some(row)) => row,
            Ok(None) => return not_found(),
            Err(err) => {
                error!(?err, "Failed to query post");
                return server_error();
            }
        };

    let ctx = state.clone().make_context(AuthClient::new(None, UserRank::Anonymous));
    let hidden = if let Some(hidden_posts) = preferences::hidden_posts(&ctx, post_statistics::post_id) {
        let post_lookup = hidden_posts.filter(post_statistics::post_id.eq(post_id));
        match diesel::select(exists(post_lookup)).first(conn.as_mut()) {
            Ok(hidden) => hidden,
            Err(err) => {
                error!(?err, "Failed to check post visibility");
                return server_error();
            }
        }
    } else {
        false
    };

    let hidden_data = EmbedData::minimal(site_name.to_string(), id);
    let hidden_title = render_template(&state.config.embed.title, &hidden_data);

    let (og_tags, page_title) = if hidden {
        let site_name_slot = render_template(&state.config.embed.site_name, &hidden_data);
        (minimal_tags(&state.config, id, &site_base, &hidden_title, &site_name_slot), hidden_title)
    } else {
        let requirements = TemplateRequirements::from_templates(&[
            &state.config.embed.site_name,
            &state.config.embed.title,
            &state.config.embed.description,
        ]);
        let data = match build_embed_data(
            &state.config,
            conn.as_mut(),
            id,
            user_id,
            post_type,
            width,
            height,
            file_size,
            source.trimmed(),
            creation_time,
            launch_date,
            &requirements,
        ) {
            Ok(data) => data,
            Err(err) => {
                error!(?err, "Failed to load embed data");
                return server_error();
            }
        };
        let site_name_slot = render_template(&state.config.embed.site_name, &data);
        let title = render_template(&state.config.embed.title, &data);
        let description = render_template(&state.config.embed.description, &data);
        let og_tags = full_tags(
            &state.config,
            id,
            safety,
            post_type,
            mime_type,
            width,
            height,
            &site_base,
            &title,
            &description,
            &site_name_slot,
        );
        (og_tags, title)
    };
    html_page(build_page(&page_title, &og_tags, &base_href(&headers), &icon_link))
}

/// Builds the meta tags shown for a visible post.
fn full_tags(
    config: &Config,
    post_id: i64,
    safety: PostSafety,
    post_type: PostType,
    mime_type: MimeType,
    width: i32,
    height: i32,
    site_base: &str,
    title: &str,
    description: &str,
    site_name_slot: &str,
) -> String {
    let og_url = absolute_url(site_base, &format!("post/{post_id}"));

    let mut tags = vec![
        format!("<meta name=\"description\" content=\"{}\"/>", encode_text(description)),
        format!("<meta property=\"og:site_name\" content=\"{}\"/>", encode_text(site_name_slot)),
        format!("<meta property=\"og:title\" content=\"{}\"/>", encode_text(title)),
        format!("<meta property=\"og:description\" content=\"{}\"/>", encode_text(description)),
        format!("<meta property=\"og:url\" content=\"{}\"/>", encode_text(&og_url)),
        format!("<link rel=\"canonical\" href=\"{}\"/>", encode_text(&og_url)),
    ];
    if config.embed.include_post_id_meta {
        tags.push(format!("<meta name=\"post-id\" content=\"{}\"/>", encode_text(&post_id.to_string())));
    }
    tags.push(format!("<meta name=\"twitter:title\" content=\"{}\"/>", encode_text(title)));
    tags.push(format!("<meta name=\"twitter:description\" content=\"{}\"/>", encode_text(description)));

    if safety == PostSafety::Unsafe && config.embed.show_rating {
        tags.push("<meta name=\"rating\" content=\"adult\"/>".to_string());
    }

    let post_hash = PostHash::new(config, post_id);
    let is_video = post_type == PostType::Video;
    tags.push(format!(
        "<meta property=\"og:type\" content=\"{}\"/>",
        if is_video { "video.other" } else { "website" }
    ));
    if is_video {
        let video_url = absolute_url(site_base, &post_hash.content_url(mime_type));
        tags.push(format!("<meta property=\"og:video\" content=\"{}\"/>", encode_text(&video_url)));
        if let Some(video_type) = video_mime_type(mime_type) {
            tags.push(format!("<meta property=\"og:video:type\" content=\"{video_type}\"/>"));
        }
    }

    match config.embed.image {
        EmbedImage::None => tags.push("<meta name=\"twitter:card\" content=\"summary\"/>".to_string()),
        EmbedImage::Thumbnail => tags.extend(thumbnail_image_tags(config, site_base, &post_hash)),
        EmbedImage::Logo if !config.embed.logo_url.is_empty() => {
            let logo_url = absolute_url(site_base, &config.embed.logo_url);
            tags.push(format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&logo_url)));
            tags.push("<meta name=\"twitter:card\" content=\"summary\"/>".to_string());
            tags.push(format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&logo_url)));
        }
        EmbedImage::Logo | EmbedImage::Post => match post_type {
            PostType::Image | PostType::Animation => {
                let image_url = absolute_url(site_base, &post_hash.content_url(mime_type));
                let twitter_card = if width >= 280 && height >= 150 { "summary_large_image" } else { "summary" };
                tags.push(format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&image_url)));
                tags.push(format!("<meta property=\"og:image:width\" content=\"{width}\"/>"));
                tags.push(format!("<meta property=\"og:image:height\" content=\"{height}\"/>"));
                tags.push(format!("<meta name=\"twitter:card\" content=\"{twitter_card}\"/>"));
                tags.push(format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&image_url)));
            }
            PostType::Video | PostType::Flash | PostType::Document => {
                tags.extend(thumbnail_image_tags(config, site_base, &post_hash));
            }
        },
    }

    tags.join("\n")
}

/// Returns the `og:image`, `twitter:image`, and `twitter:card` tags for a thumbnail preview.
fn thumbnail_image_tags(config: &Config, site_base: &str, post_hash: &PostHash) -> Vec<String> {
    let thumbnail_url = absolute_url(site_base, &post_hash.thumbnail_url());
    vec![
        format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&thumbnail_url)),
        format!("<meta property=\"og:image:width\" content=\"{}\"/>", config.thumbnails.post_width),
        format!("<meta property=\"og:image:height\" content=\"{}\"/>", config.thumbnails.post_height),
        "<meta name=\"twitter:card\" content=\"summary\"/>".to_string(),
        format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&thumbnail_url)),
    ]
}

/// Builds the minimal meta tags shown for a hidden post.
fn minimal_tags(config: &Config, post_id: i64, site_base: &str, title: &str, site_name_slot: &str) -> String {
    let og_url = absolute_url(site_base, &format!("post/{post_id}"));
    let mut tags = vec![
        format!("<meta property=\"og:site_name\" content=\"{}\"/>", encode_text(site_name_slot)),
        format!("<meta property=\"og:title\" content=\"{}\"/>", encode_text(title)),
        format!("<meta property=\"og:url\" content=\"{}\"/>", encode_text(&og_url)),
        format!("<link rel=\"canonical\" href=\"{}\"/>", encode_text(&og_url)),
    ];
    if config.embed.include_post_id_meta {
        tags.push(format!("<meta name=\"post-id\" content=\"{}\"/>", encode_text(&post_id.to_string())));
    }
    tags.push("<meta property=\"og:type\" content=\"website\"/>".to_string());
    tags.push("<meta name=\"twitter:card\" content=\"summary\"/>".to_string());
    tags.push(format!("<meta name=\"twitter:title\" content=\"{}\"/>", encode_text(title)));
    tags.join("\n")
}

/// Renders a template, substituting each `{placeholder}` with the corresponding aspect value.
fn render_template(template: &str, data: &EmbedData) -> String {
    let mut rendered = String::with_capacity(template.len());
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        rendered.push_str(&remaining[..open]);
        remaining = &remaining[open + 1..];
        let Some(close) = remaining.find('}') else {
            rendered.push('{');
            break;
        };
        rendered.push_str(&placeholder_value(&remaining[..close], data));
        remaining = &remaining[close + 1..];
    }
    rendered.push_str(remaining);
    rendered
}

/// Returns the rendered value for a single `placeholder`.
fn placeholder_value(placeholder: &str, data: &EmbedData) -> String {
    match placeholder {
        "site_name" => data.site_name.clone(),
        "post_id" => data.post_id.to_string(),
        "width" => data.width.to_string(),
        "height" => data.height.to_string(),
        "dimensions" => format!("{}x{}", data.width, data.height),
        "type" => post_type_word(data.post_type).to_string(),
        "file_size" => human_file_size(data.file_size),
        "creation_date" => data.creation_time.as_ref().map_or_else(String::new, iso_date),
        "launch_date" => data.launch_date.as_ref().map_or_else(String::new, iso_date),
        "uploader" => data.uploader.clone().unwrap_or_else(|| "Anonymous".to_string()),
        "tags" => data.all_tags.join(", "),
        "source" => data.source.clone(),
        category if category.starts_with("tags:") => data
            .tag_groups
            .get(&category["tags:".len()..])
            .map_or_else(String::new, |names| names.join(", ")),
        _ => String::new(),
    }
}

/// Holds every value that an embed template placeholder can produce.
struct EmbedData {
    post_id: i64,
    site_name: String,
    width: i32,
    height: i32,
    file_size: i64,
    post_type: PostType,
    source: String,
    creation_time: Option<DateTime>,
    launch_date: Option<DateTime>,
    uploader: Option<String>,
    all_tags: Vec<String>,
    tag_groups: HashMap<String, Vec<String>>,
}

impl EmbedData {
    /// Constructs an [`EmbedData`] with only the post id and site name populated.
    fn minimal(site_name: String, post_id: i64) -> Self {
        Self {
            post_id,
            site_name,
            width: 0,
            height: 0,
            file_size: 0,
            post_type: PostType::Image,
            source: String::new(),
            creation_time: None,
            launch_date: None,
            uploader: None,
            all_tags: Vec::new(),
            tag_groups: HashMap::new(),
        }
    }
}

/// Tracks which optional post data the embed templates reference.
#[derive(Default)]
struct TemplateRequirements {
    needs_uploader: bool,
    needs_all_tags: bool,
    tag_categories: Vec<SmallString>,
}

impl TemplateRequirements {
    fn from_templates(templates: &[&str]) -> Self {
        let mut requirements = Self::default();
        for template in templates {
            let mut remaining = *template;
            while let Some(open) = remaining.find('{') {
                remaining = &remaining[open + 1..];
                let Some(close) = remaining.find('}') else {
                    break;
                };
                let placeholder = &remaining[..close];
                match placeholder {
                    "uploader" => requirements.needs_uploader = true,
                    "tags" => requirements.needs_all_tags = true,
                    category if category.starts_with("tags:") => {
                        let category = SmallString::new(&category["tags:".len()..]);
                        if !requirements.tag_categories.iter().any(|existing| existing == &category) {
                            requirements.tag_categories.push(category);
                        }
                    }
                    _ => {}
                }
                remaining = &remaining[close + 1..];
            }
        }
        requirements
    }
}

/// Loads all values the embed templates reference, querying optional data only when needed.
fn build_embed_data(
    config: &Config,
    conn: &mut PgConnection,
    post_id: i64,
    user_id: Option<i64>,
    post_type: PostType,
    width: i32,
    height: i32,
    file_size: i64,
    source: &str,
    creation_time: DateTime,
    launch_date: Option<DateTime>,
    requirements: &TemplateRequirements,
) -> QueryResult<EmbedData> {
    let uploader = if requirements.needs_uploader {
        match user_id {
            Some(user_id) => user::table
                .select(user::name)
                .filter(user::id.eq(user_id))
                .first::<SmallString>(conn)
                .optional()?
                .map(|name| name.to_string()),
            None => None,
        }
    } else {
        None
    };

    let (all_tags, tag_groups) = if requirements.needs_all_tags || !requirements.tag_categories.is_empty() {
        let mut query = tag::table
            .inner_join(tag_category::table)
            .inner_join(tag_name::table)
            .inner_join(post_tag::table)
            .filter(post_tag::post_id.eq(post_id))
            .filter(TagName::primary())
            .select((tag_category::name, tag_name::name))
            .order((tag_category::order, tag_name::name))
            .into_boxed();
        if !requirements.needs_all_tags {
            query = query.filter(tag_category::name.eq_any(requirements.tag_categories.clone()));
        }
        let rows = query.load::<(SmallString, SmallString)>(conn)?;
        let mut all_tags = Vec::new();
        let mut tag_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (category, name) in rows {
            if all_tags.len() < MAX_TAGS_PER_SLOT {
                all_tags.push(name.to_string());
            }
            let names = tag_groups.entry(category.to_string()).or_default();
            if names.len() < MAX_TAGS_PER_SLOT {
                names.push(name.to_string());
            }
        }
        (all_tags, tag_groups)
    } else {
        (Vec::new(), HashMap::new())
    };

    Ok(EmbedData {
        post_id,
        site_name: config.public_info.name.to_string(),
        width,
        height,
        file_size,
        post_type,
        source: source.trim().to_string(),
        creation_time: Some(creation_time),
        launch_date,
        uploader,
        all_tags,
        tag_groups,
    })
}

/// Returns the lowercase word for `post_type`.
fn post_type_word(post_type: PostType) -> &'static str {
    match post_type {
        PostType::Image => "image",
        PostType::Animation => "animation",
        PostType::Video => "video",
        PostType::Flash => "flash",
        PostType::Document => "document",
    }
}

/// Formats `bytes` using binary units, matching the `ByteSize` display in the API.
#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn human_file_size(bytes: i64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else if value.fract() == 0.0 {
        format!("{} {}", value as u64, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// Formats `value` as an ISO `YYYY-MM-DD` date, or an empty string on error.
fn iso_date(value: &DateTime) -> String {
    value.format(&time::format_description::well_known::Iso8601::DATE).unwrap_or_default()
}

/// Returns the favicon `<link>` element for the embed page, using the configured logo if set.
fn icon_link(logo_url: &str, site_base: &str) -> String {
    let href = if logo_url.is_empty() {
        "img/favicon.png".to_string()
    } else {
        absolute_url(site_base, logo_url)
    };
    format!("<link rel='shortcut icon' type='image/png' href='{}'/>", encode_text(&href))
}

/// Returns the `og:video:type` for `mime_type`, if a known video type exists.
fn video_mime_type(mime_type: MimeType) -> Option<&'static str> {
    match mime_type {
        MimeType::Mp4 => Some("video/mp4"),
        MimeType::Webm => Some("video/webm"),
        MimeType::Mov => Some("video/quicktime"),
        _ => None,
    }
}

/// Returns the base URL for absolute embed links, derived from config or request headers.
fn site_base_url(config: &Config, host: Option<&str>, headers: &HeaderMap) -> String {
    if let Some(domain) = config.domain.as_deref() {
        return domain.trim_end_matches('/').to_string();
    }
    if config.trusted_proxy
        && let Some(proto) = headers.get("x-forwarded-proto").and_then(|value| value.to_str().ok())
        && let Some(host) = host
    {
        return format!("{proto}://{host}");
    }
    host.map_or_else(String::new, |host| format!("http://{host}"))
}

/// Prepends `base` to `path_or_url` unless it is already absolute or base is empty.
fn absolute_url(base: &str, path_or_url: &str) -> String {
    let is_absolute = path_or_url.starts_with("http://") || path_or_url.starts_with("https://");
    if is_absolute || base.is_empty() {
        path_or_url.to_string()
    } else {
        format!("{base}/{path_or_url}")
    }
}

/// Returns the base href for the SPA assets, derived from the original request URI.
/// Only accepts path-only prefixes to prevent scheme injection from an untrusted header.
fn base_href(headers: &HeaderMap) -> String {
    let Some(original_uri) = headers.get("x-original-uri").and_then(|value| value.to_str().ok()) else {
        return "/".to_string();
    };
    let Some(capture) = ORIGINAL_URI_REGEX.captures(original_uri).and_then(|captures| captures.get(1)) else {
        return "/".to_string();
    };
    let prefix = capture.as_str();
    if prefix.starts_with('/') && !prefix.contains(':') {
        format!("{prefix}/")
    } else {
        "/".to_string()
    }
}

/// Replaces the placeholders in the page template.
fn build_page(title: &str, og_tags: &str, base_href: &str, icon_link: &str) -> String {
    PAGE_TEMPLATE
        .replace("%%TITLE%%", &encode_text(title))
        .replace("%%OG_TAGS%%", og_tags)
        .replace("%%BASE_HREF%%", &encode_text(base_href))
        .replace("%%ICON_LINK%%", icon_link)
}

fn html_page(html: String) -> Response {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
            (CACHE_CONTROL, HeaderValue::from_static("public, max-age=300")),
        ],
        html,
    )
        .into_response()
}

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

fn server_error() -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config;
    use crate::test::*;
    use axum::ServiceExt;
    use axum::extract::Request;
    use axum_test::TestServer;
    use serial_test::serial;
    use std::sync::Arc;

    fn server() -> TestServer {
        TestServer::new(ServiceExt::<Request>::into_make_service(routes(get_state()))).unwrap()
    }

    fn server_with_config(relative_path: &str) -> TestServer {
        let mut state = get_state();
        let mut config = config::test_config(Some(relative_path));
        config.data_dir = state.config.data_dir.clone();
        state.config = Arc::new(config);
        TestServer::new(ServiceExt::<Request>::into_make_service(routes(state))).unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn image_post() {
        let server = server();
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert!(response.header("content-type").to_str().unwrap().starts_with("text/html"));
        let body = response.text();
        assert!(body.contains("og:title\" content=\"Post #1 - oxibooru\""));
        assert!(body.contains("property=\"og:type\" content=\"website\""));
        assert!(body.contains("property=\"og:image\""));
        assert!(body.contains("name=\"post-id\" content=\"1\""));
        assert!(body.contains("name=\"twitter:card\" content=\"summary_large_image\""));
        assert!(body.contains("rel=\"canonical\""));
        assert!(body.contains("content=\"View this 1000x2000 image\""));
        assert!(!body.contains("og:video"));
        assert!(body.contains("<base href='/'/>"));
    }

    #[tokio::test]
    #[serial]
    async fn video_post() {
        let server = server();
        let response = server.get("/embed/post/5").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("property=\"og:type\" content=\"video.other\""));
        assert!(body.contains("property=\"og:video\""));
        assert!(body.contains("name=\"rating\" content=\"adult\""));
    }

    #[tokio::test]
    #[serial]
    async fn hidden_post() {
        let server = server_with_config("embed/hidden");
        let response = server.get("/embed/post/5").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("og:title\" content=\"Post #5 - oxibooru\""));
        assert!(body.contains("name=\"post-id\" content=\"5\""));
        assert!(!body.contains("og:image"));
        assert!(!body.contains("og:description"));
        assert!(!body.contains("rating"));
        assert!(!body.contains("og:video"));
        assert!(!body.contains("twitter:image"));
    }

    #[tokio::test]
    #[serial]
    async fn not_found() {
        let server = server();
        let response = server.get("/embed/post/999999").await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[serial]
    async fn disabled() {
        let server = server_with_config("embed/disabled");
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(!body.contains("og:title"));
        assert!(!body.contains("og:image"));
    }

    #[tokio::test]
    #[serial]
    async fn unsafe_base_href_is_rejected() {
        let server = server();
        let response = server
            .get("/embed/post/1")
            .add_header("x-original-uri", "javascript:alert(1)/post/1")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("<base href='/'/>"));
    }

    #[tokio::test]
    #[serial]
    async fn subpath_base_href() {
        let server = server();
        let response = server
            .get("/embed/post/1")
            .add_header("x-original-uri", "/booru/post/1")
            .await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("<base href='/booru/'/>"));
    }

    #[tokio::test]
    #[serial]
    async fn custom_templates() {
        let server = server_with_config("embed/custom");
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("og:title\" content=\"1 upload\""));
        assert!(body.contains("og:description\" content=\"1000x2000 image by restricted_user\""));
        assert!(!body.contains("Post #1 - oxibooru"));
        assert!(!body.contains("View this 1000x2000 image"));
    }

    #[tokio::test]
    #[serial]
    async fn tag_category_placeholder() {
        let server = server_with_config("embed/tag_category");
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("og:description\" content=\"shakespeare\""));
    }

    #[tokio::test]
    #[serial]
    async fn logo_image() {
        let server = server_with_config("embed/logo");
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        let og_image = body.split("og:image\" content=\"").nth(1).unwrap_or_default();
        assert!(og_image.contains("/img/logo.png"));
        let favicon = body.split("<link rel='shortcut icon'").nth(1).unwrap_or_default();
        assert!(favicon.contains("logo.png"));
        assert!(!body.contains("summary_large_image"));
    }

    #[tokio::test]
    #[serial]
    async fn no_image() {
        let server = server_with_config("embed/no_image");
        let response = server.get("/embed/post/1").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(!body.contains("og:image"));
        assert!(!body.contains("twitter:image"));
        assert!(body.contains("twitter:card\" content=\"summary\""));
    }

    #[tokio::test]
    #[serial]
    async fn full_custom() {
        let server = server_with_config("embed/full_custom");
        let response = server.get("/embed/post/5").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.text();
        assert!(body.contains("og:title\" content=\"oxibooru | 5\""));
        assert!(body.contains("100 GiB"));

        let mut conn = get_connection().unwrap();
        let creation_time: DateTime = post::table.find(5).select(post::creation_time).first(&mut conn).unwrap();
        let creation_date = creation_time.format(&time::format_description::well_known::Iso8601::DATE).unwrap();
        assert!(body.contains(&creation_date));
    }
}
