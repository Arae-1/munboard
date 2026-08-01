# legacy.rs — Legacy URL Redirect Endpoint

## Purpose

This file provides a single HTTP API handler that redirects old-style content URLs to the new hashed content URL format. It validates the legacy hash using HMAC with BLAKE3.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::app::AppState` | Application state |
| `crate::config::Config` | Server configuration |
| `crate::content::hash::PostHash` | New-style post content hash |
| `crate::model::enums::MimeType` | MIME type enum |
| `axum::extract::{Path, State}` | Axum extractors |
| `axum::http::StatusCode` | HTTP status codes |
| `axum::response::Redirect` | HTTP redirect response |
| `base64::Engine` | Base64 encoding/decoding |
| `base64::engine::general_purpose::URL_SAFE_NO_PAD` | URL-safe Base64 without padding |
| `hmac::{KeyInit, Mac, SimpleHmac}` | HMAC computation |
| `utoipa_axum::router::OpenApiRouter` | Axum router |

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with a single route at `/legacy/redirect/{filename}` using the `GET` method handled by `redirect`.

---

## Endpoint: Legacy Redirect

- **Method:** GET
- **Path:** `/legacy/redirect/{filename}`
- **Authentication:** None

### Handler: `redirect`

```
async fn redirect(State(state): State<AppState>, Path(filename): Path<String>) -> Result<Redirect, StatusCode>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `filename` | `String` | Path | Legacy filename in the format `{post_id}_{hash}.{extension}` |

### Request Body

None.

### Response

| Status | Description |
|--------|-------------|
| 301 | Permanent redirect to the new content URL |
| 404 | Legacy hash does not match the computed hash |
| 415 | Unsupported file extension (cannot map to `MimeType`) |
| 422 | Invalid filename format |

### Error Responses (Non-Redirect)

| Status | Condition |
|--------|-----------|
| 422 | Filename does not contain `_` separator |
| 422 | `post_id` portion is not a valid `i64` |
| 422 | Filename does not contain `.` separator |
| 415 | File extension cannot be parsed as a `MimeType` |
| 404 | Legacy URL hash does not match the computed HMAC |

### Handler Logic Step by Step

1. Split the `filename` by the first underscore (`_`):
   - Left side is `post_id` as a string.
   - Right side is `tail` containing the hash and extension.
   - If no underscore is found, return `StatusCode::UNPROCESSABLE_ENTITY` (422).
2. Parse `post_id` as `i64`:
   - If parsing fails, return `StatusCode::UNPROCESSABLE_ENTITY` (422).
3. Split `tail` by the last dot (`.`):
   - Left side is `post_hash` (the legacy hash string).
   - Right side is `extension`.
   - If no dot is found, return `StatusCode::UNPROCESSABLE_ENTITY` (422).
4. Parse the extension into a `MimeType`:
   - Call `MimeType::from_extension(extension)`.
   - If this fails (returns `Err`), return `StatusCode::UNSUPPORTED_MEDIA_TYPE` (415).
5. Compute the expected legacy hash:
   - Call `legacy_url_hash(&state.config, post_id)`.
   - Compare with the `post_hash` from the URL.
   - If they do not match, return `StatusCode::NOT_FOUND` (404).
6. Compute the new-style `PostHash`:
   - Call `PostHash::new(&state.config, post_id)`.
7. Generate the new content URL:
   - Call `new_post_hash.content_url(mime_type)`.
   - Prepend a `/`.
8. Return `Redirect::permanent(&new_url)` (HTTP 301).

### Helper Function: `legacy_url_hash`

```
fn legacy_url_hash(config: &Config, post_id: i64) -> String
```

Type alias: `Hmac = SimpleHmac<blake3::Hasher>`

1. Create a new HMAC instance keyed with `config.content_secret` bytes.
2. Feed the `post_id` as little-endian bytes into the HMAC.
3. Finalize and encode the result as URL-safe Base64 without padding.
4. Return the encoded hash string.

## Security

The legacy hash uses HMAC-BLAKE3 with the server's `content_secret`. An attacker cannot guess a valid legacy URL without knowing the secret. This prevents enumeration of post IDs through the legacy redirect endpoint.
