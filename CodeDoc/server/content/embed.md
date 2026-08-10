# OpenGraph Embed Pages (`embed.rs`)

## Summary

This module serves small HTML pages with OpenGraph and Twitter meta tags for post URLs. Chat applications (Discord, Telegram) fetch shared links server-side to render rich embeds. They do not run JavaScript, so the SPA client cannot provide the embed. This module serves a small HTML page for each post so crawlers can render a preview. The page also boots the SPA in browsers.

## Module Purpose

The module lives in `server/src/embed.rs` and is declared in `main.rs` as `mod embed;`. It is a top-level server module, not part of the `content` processing pipeline. The documentation lives here because the pages describe post content.

The embed router is a plain axum router, not part of the `OpenAPI` JSON API router, because it responds with HTML. `app::run` merges `embed::routes(state)` into the main application router.

## Routes

| Method | Path | Handler | Purpose |
|--------|------|---------|---------|
| `GET` | `/embed/post/{id}` | `post_page` | Serves the OpenGraph embed page for one post |

The deployment proxy must rewrite post URLs to this route. nginx (`client/nginx.conf.docker`) proxies `^/post/[0-9]+/?$` to `/embed/post/{id}` and forwards the original URI in the `X-Original-URI` header. The embed router is merged into the main router before the normalized API fallback, so trailing-slash normalization does not apply to it.

## Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `PAGE_TEMPLATE` | `include_str!("embed/page.html")` | HTML shell with `%%TITLE%%`, `%%OG_TAGS%%`, `%%BASE_HREF%%`, and `%%ICON_LINK%%` placeholders |
| `ORIGINAL_URI_REGEX` | `^(.+?)/post/[0-9]+/?$` | Matches the original request URI, capturing any subdirectory prefix |
| `MAX_TAGS_PER_SLOT` | `10` | Maximum number of tag names rendered in one `{tags}` or `{tags:<category>}` slot |

`PostEmbedRow` is a type alias for the row fetched from the `post` table: `(id, user_id, safety, type_, mime_type, width, height, file_size, creation_time, launch_date, source)`.

## Function: `routes(state: AppState) -> Router`

```rust
pub fn routes(state: AppState) -> Router
```

Returns a router serving OpenGraph embed pages. Registers `GET /embed/post/{id}` with the `post_page` handler and attaches the state. This is a plain axum router, not part of the `OpenAPI` JSON API router, because it responds with HTML.

## Function: `post_page`

```rust
async fn post_page(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(post_id): Path<i64>,
) -> Response
```

**Algorithm:**

1. If `config.enable_embeds` is false, return a plain page with no meta tags so browsers still work. The page uses `build_page(site_name, "", "/", &icon_link("", ""))`.
2. Read the `host` header. Compute the site base URL with `site_base_url` and the favicon link with `icon_link`.
3. Get a database connection from the pool. On failure, log and return a 500 response.
4. Query the `post` row by id. If no row exists, return a 404 response.
5. Build an anonymous client context and check hidden status through `preferences::hidden_posts`. The anonymous tag and safety hiding preferences apply.
6. If the post is hidden, build `EmbedData::minimal`, render the `config.embed.title` and `config.embed.site_name` templates, and produce tags with `minimal_tags`.
7. Otherwise, scan the site_name, title, and description templates with `TemplateRequirements::from_templates`, load the referenced data with `build_embed_data`, render all three template slots, and produce tags with `full_tags`.
8. Return `build_page(&page_title, &og_tags, &base_href(&headers), &icon_link)` wrapped by `html_page`.

`html_page` returns status 200 with `Content-Type: text/html; charset=utf-8` and `Cache-Control: public, max-age=300`.

## Function: `full_tags`

```rust
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
) -> String
```

Builds the meta tags shown for a visible post. The tags are joined with newlines.

**Always emitted:**
- `<meta name="description">`
- `<meta property="og:site_name">`
- `<meta property="og:title">`
- `<meta property="og:description">`
- `<meta property="og:url">` (the post URL via `absolute_url`)
- `<link rel="canonical">` (same URL)
- `<meta name="twitter:title">`
- `<meta name="twitter:description">`

**Conditional tags:**
- If `config.embed.include_post_id_meta`, emit `<meta name="post-id">`.
- If the post is `Unsafe` and `config.embed.show_rating`, emit `<meta name="rating" content="adult"/>`.
- `og:type` is `video.other` for video posts, `website` otherwise.
- Video posts also emit `og:video`, `og:video:secure_url` (the content URL), `og:video:type` when `video_mime_type` knows it, and `og:video:width`/`og:video:height`.
- The image tags depend on `config.embed.image` (see the image mode table below).

## Function: `minimal_tags`

```rust
fn minimal_tags(config: &Config, post_id: i64, site_base: &str, title: &str, site_name_slot: &str) -> String
```

Builds the minimal meta tags shown for a hidden post:
- `og:site_name`, `og:title`, `og:url`, and the canonical link.
- `<meta name="post-id">` if `config.embed.include_post_id_meta`.
- `og:type` is `website` and `twitter:card` is `summary`.
- `twitter:title`.

A hidden post emits no `og:image`, no `og:description`, no `rating`, and no `og:video`, so the post content is not revealed to crawlers.

## Function: `thumbnail_image_tags(config, site_base, post_hash) -> Vec<String>`

Returns the `og:image`, `twitter:image`, and `twitter:card` tags for a thumbnail preview:
- `og:image` is the thumbnail URL from `PostHash::thumbnail_url()`.
- `og:image:width` and `og:image:height` come from `config.thumbnails.post_width` and `config.thumbnails.post_height`.
- `twitter:card` is `summary`.
- `twitter:image` is the same thumbnail URL.

## Behavior Per Embed Image Mode

`config.embed.image` selects the preview image tags in `full_tags`:

| Mode | Image tags |
|------|------------|
| `None` | No image tags. `twitter:card` is `summary`. |
| `Thumbnail` | Always `thumbnail_image_tags` (thumbnail URL, thumbnail dimensions, `summary` card). |
| `Logo` with a non-empty `logo_url` | `og:image` and `twitter:image` are the logo URL. `twitter:card` is `summary`. |
| `Logo` with an empty `logo_url` | Falls through to the `Post` behavior (the guard `!config.embed.logo_url.is_empty()` does not match). |
| `Post` for `Image` or `Animation` posts | `og:image` is the content URL. `og:image:width` and `og:image:height` are the post dimensions. `twitter:card` is `summary_large_image` when `width >= 280` and `height >= 150`, otherwise `summary`. `twitter:image` is the content URL. |
| `Post` for `Video`, `Flash`, or `Document` posts | `thumbnail_image_tags` (the thumbnail preview). |

## Function: `icon_link(logo_url, site_base) -> String`

Returns the favicon `<link>` element for the embed page:
- If `logo_url` is empty, the href is `img/favicon.png`.
- Otherwise, the href is `absolute_url(site_base, logo_url)`.

The result is `<link rel='shortcut icon' type='image/png' href='...'/>`. The href passes through `encode_html`.

## Function: `video_mime_type(mime_type: MimeType) -> Option<&'static str>`

Returns the `og:video:type` for a MIME type, if a known video type exists:

| MimeType | og:video:type |
|----------|---------------|
| `Mp4` | `video/mp4` |
| `Webm` | `video/webm` |
| `Mov` | `video/quicktime` |
| other | `None` |

## Function: `encode_html(value: &str) -> String`

Escapes `value` for inclusion in the embed page: the standard HTML entities plus single and double quotes and backticks, so the same encoding is safe in both element text and quoted attribute values. `encode_text` leaves quotes intact, which would allow breaking out of the `content="..."` attributes.

```rust
fn encode_html(value: &str) -> String {
    encode_quoted_attribute(value).replace('`', "&#96;")
}
```

## Function: `build_page(title, og_tags, base_href, icon_link) -> String`

Replaces the placeholders in `PAGE_TEMPLATE`:

| Placeholder | Replacement |
|-------------|-------------|
| `%%TITLE%%` | The page title, passed through `encode_html` |
| `%%OG_TAGS%%` | The rendered meta tag string (already escaped) |
| `%%BASE_HREF%%` | The base href, passed through `encode_html` |
| `%%ICON_LINK%%` | The favicon link element (already escaped) |

The template (`embed/page.html`) is an HTML shell that loads `css/app.min.css`, `css/vendor.min.css`, `js/vendor.min.js`, `js/app.min.js`, and the Ruffle Flash player. It contains empty `#top-navigation-holder` and `#content-holder` divs that the SPA populates. The page boots the SPA in a real browser.

## Function: `base_href(headers: &HeaderMap) -> String`

Returns the base href for the SPA assets, derived from the original request URI. The `X-Original-URI` header is set by the nginx proxy. `ORIGINAL_URI_REGEX` captures any subdirectory prefix so relative asset links keep working when the SPA is served from a subpath (for example `/booru/post/123` gives `/booru/`).

Only path-only prefixes are accepted. A prefix must start with `/` and must not contain `:`. This prevents scheme injection from an untrusted header. If the header is absent, the URI does not match, or the prefix is rejected, the function returns `/`.

## Function: `absolute_url(base: &str, path_or_url: &str) -> String`

Prepends `base` to `path_or_url` unless it is already absolute or base is empty. A value is absolute when it starts with `http://` or `https://`.

## Function: `site_base_url(config, host, headers) -> String`

Returns the base URL for absolute embed links:
1. If `config.domain` is set, use it with trailing slashes trimmed.
2. Else if `config.trusted_proxy` is true and the `x-forwarded-proto` and `host` headers exist, use `{proto}://{host}`.
3. Else use `http://{host}`, or an empty string when no host header exists.

## Template Placeholders

The embed templates (`config.embed.site_name`, `config.embed.title`, `config.embed.description`) accept the following placeholders:

| Placeholder | Value |
|-------------|-------|
| `{site_name}` | The site name from `config.public_info.name` |
| `{post_id}` | The post id |
| `{width}` | The post width |
| `{height}` | The post height |
| `{dimensions}` | `{width}x{height}` |
| `{type}` | The lowercase post type word: `image`, `animation`, `video`, `flash`, or `document` |
| `{file_size}` | The file size in binary units, matching the `ByteSize` display in the API |
| `{creation_date}` | The creation date as ISO `YYYY-MM-DD`, or empty on error |
| `{launch_date}` | The launch date as ISO `YYYY-MM-DD`, or empty when not set |
| `{uploader}` | The uploader user name, or `Anonymous` |
| `{tags}` | The primary tag names joined with `", "`, up to `MAX_TAGS_PER_SLOT` |
| `{tags:<category>}` | The primary tag names of one category, joined with `", "`, up to `MAX_TAGS_PER_SLOT` |
| `{source}` | The post source, trimmed |

Unknown placeholders render as an empty string. The default templates are `site_name = "{site_name}"`, `title = "Post #{post_id} - {site_name}"`, and `description = "View this {width}x{height} {type}"`.

## Template Rendering

`render_template(template, data)` scans a template for `{placeholder}` pairs and substitutes each one with `placeholder_value(placeholder, data)`. An unclosed `{` renders literally.

`EmbedData` holds every value a placeholder can produce. `EmbedData::minimal` builds the value set for a hidden post, with only the post id and site name populated.

`TemplateRequirements::from_templates` scans the site_name, title, and description templates and records which optional post data they reference: the uploader, all tags, and specific tag categories. `build_embed_data` then queries optional data only when the templates reference it, so a plain default config loads no extra data. Tag queries select primary tag names ordered by tag category order and tag name, and the uploader query reads `user::name`.

## Supporting Functions

| Function | Purpose |
|----------|---------|
| `post_type_word(post_type)` | Returns the lowercase word for a post type |
| `human_file_size(bytes)` | Formats bytes with binary units (`B`, `KiB`, `MiB`, `GiB`, `TiB`, `PiB`) |
| `iso_date(value)` | Formats a `DateTime` as ISO `YYYY-MM-DD`, or empty on error |
| `html_page(html)` | Wraps HTML in a 200 response with `text/html; charset=utf-8` and cache headers |
| `not_found()` | Returns a 404 response with body `Not Found` |
| `server_error()` | Returns a 500 response with body `Internal Server Error` |

## Security Notes

- All values inserted into `content="..."` attributes pass through `encode_html`, which escapes quotes and backticks in addition to the standard HTML entities. This prevents breaking out of quoted attributes and blocks stored XSS through post titles, descriptions, sources, tag names, and user names.
- `build_page` also passes `%%TITLE%%` and `%%BASE_HREF%%` through `encode_html`.
- `base_href` accepts only path-only prefixes from the `X-Original-URI` header. Values that do not start with `/` or that contain `:` are rejected, which prevents scheme injection such as `javascript:`.
- A missing post returns 404. A database failure returns 500. No partial content is served.
- Hidden posts receive `minimal_tags` only. The page does not reveal the image, description, rating, or video data of a hidden post.
- The `og:url` and canonical links use the site base from `config.domain` when configured, so embed URLs stay under the site's own domain.

## Configuration

The `[embed]` section of the configuration controls the embed appearance: `logo_url`, `image`, `site_name`, `title`, `description`, `show_rating`, and `include_post_id_meta`. `enable_embeds` turns the pages on and off. `config.domain` and `config.trusted_proxy` control the absolute URL base. See [core/config.md](../core/config.md) for the full configuration reference.
