//! Serves HTML pages with OpenGraph/Twitter meta tags for post URLs.
//!
//! Chat applications (Discord, Telegram) fetch shared links server-side to
//! render rich embeds. They do not run JavaScript, so the SPA client cannot
//! provide the embed. This module serves a small HTML page for each post so
//! crawlers can render a preview. The page also boots the SPA in browsers.

use crate::app::AppState;
use crate::auth::Client as AuthClient;
use crate::config::Config;
use crate::content::hash::PostHash;
use crate::model::enums::{MimeType, PostSafety, PostType, UserRank};
use crate::schema::{post, post_statistics};
use crate::search::preferences;
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
use std::sync::LazyLock;
use tracing::error;

/// HTML shell template with `%%TITLE%%`, `%%OG_TAGS%%`, and `%%BASE_HREF%%` placeholders.
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

async fn post_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
) -> Response {
    let site_name: &str = &state.config.public_info.name;

    // Serve a plain page when embeds are disabled so browsers still work.
    if !state.config.enable_embeds {
        return html_page(build_page(site_name, "", "/"));
    }

    let host = headers.get("host").and_then(|value| value.to_str().ok());
    let site_base = site_base_url(&state.config, host, &headers);
    let title = format!("Post #{post_id} - {site_name}");

    let mut conn = match state.connection_pool.get().await {
        Ok(conn) => conn,
        Err(err) => {
            error!(?err, "Failed to get database connection");
            return server_error();
        }
    };

    let (id, safety, post_type, mime_type, width, height, _file_size): (
        i64,
        PostSafety,
        PostType,
        MimeType,
        i32,
        i32,
        i64,
    ) = match post::table
            .find(post_id)
            .select((
                post::id,
                post::safety,
                post::type_,
                post::mime_type,
                post::width,
                post::height,
                post::file_size,
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

    let og_tags = if hidden {
        minimal_tags(&state.config, post_id, &site_base, &title)
    } else {
        full_tags(
            &state.config,
            id,
            safety,
            post_type,
            mime_type,
            width,
            height,
            &site_base,
            &title,
        )
    };
    html_page(build_page(&title, &og_tags, &base_href(&headers)))
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
) -> String {
    let site_name: &str = &config.public_info.name;
    let description = description(post_type, width, height);
    let og_url = absolute_url(site_base, &format!("post/{post_id}"));

    let mut tags = vec![
        format!("<meta name=\"description\" content=\"{}\"/>", encode_text(&description)),
        format!("<meta property=\"og:site_name\" content=\"{}\"/>", encode_text(site_name)),
        format!("<meta property=\"og:title\" content=\"{}\"/>", encode_text(title)),
        format!("<meta property=\"og:description\" content=\"{}\"/>", encode_text(&description)),
        format!("<meta property=\"og:url\" content=\"{}\"/>", encode_text(&og_url)),
        format!("<link rel=\"canonical\" href=\"{}\"/>", encode_text(&og_url)),
        format!("<meta name=\"post-id\" content=\"{}\"/>", encode_text(&post_id.to_string())),
        format!("<meta name=\"twitter:title\" content=\"{}\"/>", encode_text(title)),
        format!("<meta name=\"twitter:description\" content=\"{}\"/>", encode_text(&description)),
    ];

    if safety == PostSafety::Unsafe {
        tags.push("<meta name=\"rating\" content=\"adult\"/>".to_string());
    }

    let post_hash = PostHash::new(config, post_id);
    match post_type {
        PostType::Image | PostType::Animation => {
            let image_url = absolute_url(site_base, &post_hash.content_url(mime_type));
            let twitter_card = if width >= 280 && height >= 150 { "summary_large_image" } else { "summary" };
            tags.push("<meta property=\"og:type\" content=\"website\"/>".to_string());
            tags.push(format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&image_url)));
            tags.push(format!("<meta property=\"og:image:width\" content=\"{width}\"/>"));
            tags.push(format!("<meta property=\"og:image:height\" content=\"{height}\"/>"));
            tags.push(format!("<meta name=\"twitter:card\" content=\"{twitter_card}\"/>"));
            tags.push(format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&image_url)));
        }
        PostType::Video => {
            let video_url = absolute_url(site_base, &post_hash.content_url(mime_type));
            let thumbnail_url = absolute_url(site_base, &post_hash.thumbnail_url());
            tags.push("<meta property=\"og:type\" content=\"video.other\"/>".to_string());
            tags.push(format!("<meta property=\"og:video\" content=\"{}\"/>", encode_text(&video_url)));
            if let Some(video_type) = video_mime_type(mime_type) {
                tags.push(format!("<meta property=\"og:video:type\" content=\"{video_type}\"/>"));
            }
            tags.push(format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&thumbnail_url)));
            tags.push(format!("<meta property=\"og:image:width\" content=\"{}\"/>", config.thumbnails.post_width));
            tags.push(format!("<meta property=\"og:image:height\" content=\"{}\"/>", config.thumbnails.post_height));
            tags.push("<meta name=\"twitter:card\" content=\"summary\"/>".to_string());
            tags.push(format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&thumbnail_url)));
        }
        PostType::Flash | PostType::Document => {
            let thumbnail_url = absolute_url(site_base, &post_hash.thumbnail_url());
            tags.push("<meta property=\"og:type\" content=\"website\"/>".to_string());
            tags.push(format!("<meta property=\"og:image\" content=\"{}\"/>", encode_text(&thumbnail_url)));
            tags.push(format!("<meta property=\"og:image:width\" content=\"{}\"/>", config.thumbnails.post_width));
            tags.push(format!("<meta property=\"og:image:height\" content=\"{}\"/>", config.thumbnails.post_height));
            tags.push("<meta name=\"twitter:card\" content=\"summary\"/>".to_string());
            tags.push(format!("<meta name=\"twitter:image\" content=\"{}\"/>", encode_text(&thumbnail_url)));
        }
    }

    tags.join("\n")
}

/// Builds the minimal meta tags shown for a hidden post.
fn minimal_tags(config: &Config, post_id: i64, site_base: &str, title: &str) -> String {
    let site_name: &str = &config.public_info.name;
    let og_url = absolute_url(site_base, &format!("post/{post_id}"));
    let tags = [
        format!("<meta property=\"og:site_name\" content=\"{}\"/>", encode_text(site_name)),
        format!("<meta property=\"og:title\" content=\"{}\"/>", encode_text(title)),
        format!("<meta property=\"og:url\" content=\"{}\"/>", encode_text(&og_url)),
        format!("<link rel=\"canonical\" href=\"{}\"/>", encode_text(&og_url)),
        format!("<meta name=\"post-id\" content=\"{}\"/>", encode_text(&post_id.to_string())),
        "<meta property=\"og:type\" content=\"website\"/>".to_string(),
        "<meta name=\"twitter:card\" content=\"summary\"/>".to_string(),
        format!("<meta name=\"twitter:title\" content=\"{}\"/>", encode_text(title)),
    ];
    tags.join("\n")
}

/// Returns the embed description for `post_type`.
fn description(post_type: PostType, width: i32, height: i32) -> String {
    match post_type {
        PostType::Image | PostType::Animation => format!("View this {width}x{height} image"),
        PostType::Video => format!("View this {width}x{height} video"),
        PostType::Flash | PostType::Document => format!("View this {width}x{height} file"),
    }
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
fn build_page(title: &str, og_tags: &str, base_href: &str) -> String {
    PAGE_TEMPLATE
        .replace("%%TITLE%%", &encode_text(title))
        .replace("%%OG_TAGS%%", og_tags)
        .replace("%%BASE_HREF%%", &encode_text(base_href))
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
}
