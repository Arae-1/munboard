# upload.rs — Upload API Endpoints

## Purpose

This file provides the HTTP API handler for temporary file uploads. Uploaded files receive a token that can be used in other requests. Files are deleted after a short time.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::UPLOAD_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::app::{AppState, Context}` | Application state and request context |
| `crate::config::{Action, Config}` | Privilege actions and configuration |
| `crate::content::upload::{PartName, UploadToken}` | Upload types |
| `crate::content::{download, upload}` | Download and upload functions |
| `crate::extract::{Ctx, Json, JsonOrMultipart}` | Axum extractors |
| `serde::{Deserialize, Serialize}` | Serialization |
| `url::Url` | URL type for content URL |
| `utoipa::ToSchema` | OpenAPI schema |
| `utoipa_axum` | OpenAPI router |

## Router

### `routes(config: &Config) -> OpenApiRouter<AppState>`

Returns a router with the `upload` route and a `DefaultBodyLimit::max(config.max_multipart_body_size())` layer that limits the total request body size. Individual file parts are limited to `config.max_upload_size`.

---

## Request Body Types

### `UploadBody` (JSON)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contentUrl` | `Url` | Yes | URL to fetch content from |

### `MultipartUpload` (multipart/form-data)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `metadata` | `UploadBody` | No | JSON metadata with `contentUrl` |
| `content` | binary | No | The actual file content |

## Response Type

### `UploadResponse`

| Field | Type | Description |
|-------|------|-------------|
| `token` | `UploadToken` | Token to reference this upload in other requests |

---

## Endpoint: Upload (Temporary File)

- **Method:** POST
- **Path:** `/uploads`
- **Authentication:** Required
- **Privilege:** `Action::UploadCreate` for all requests; URL downloads additionally require `Action::UploadUseDownloader`
- **Per-file size limit:** `config.max_upload_size`
- **Total body size limit:** `config.max_multipart_body_size()` (2x `max_upload_size` + 2 MiB)

### Handler: `upload`

```
async fn upload(ctx: Ctx, body: JsonOrMultipart<UploadBody>) -> ApiResult<Json<UploadResponse>>
```

### Request Content Types

The endpoint accepts two content types:
- `application/json` — the body is a JSON `UploadBody` with a `contentUrl` field
- `multipart/form-data` — the body is a multipart form with either:
  - A `content` file part (binary data), or
  - A `metadata` part containing a JSON `UploadBody`, or
  - Both

### Request Parameters

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UploadResponse` | Upload token |
| 403 | `ErrorResponse` | Insufficient privileges |

### Handler Logic Step by Step

1. Verify the authenticated user has `Action::UploadCreate` privilege.
2. Match on the body type:

   **JSON branch** (`JsonOrMultipart::Json(payload)`):
   - Call `upload_from_url(&ctx, payload)`.
   - URL downloads require the `Action::UploadUseDownloader` privilege, verified before any network fetch.
   - The function calls `download::from_url(config, body.content_url)` which downloads the file from the given URL and stores it in temporary storage.
   - Return `UploadResponse` with the token.

   **Multipart branch** (`JsonOrMultipart::Multipart(payload)`):
   - Call `upload::extract(&ctx.config, payload, [PartName::Content])`.
   - This extracts file parts named `content` and a possible `metadata` part from the multipart form.
   - Three sub-cases:

     a. If `decoded_body.files` matches `[Some(token)]` (a file with name `content` was uploaded directly):
        - Return `UploadResponse { token }`.
        - This direct file-upload path is gated by `Action::UploadCreate` only.

     b. If `decoded_body.metadata` is `Some(metadata)` (a JSON metadata part was provided):
        - Deserialize the metadata bytes as `UploadBody` via `serde_json::from_slice`.
        - Call `upload_from_url` with the URL from the metadata.
        - This allows URL-based uploads through the multipart form, and therefore also requires `Action::UploadUseDownloader`.

     c. If neither a file nor metadata is present:
        - Return `ApiError::MissingFormData`.

### Helper Function: `upload_from_url`

```
async fn upload_from_url(ctx: &Context, body: UploadBody) -> ApiResult<Json<UploadResponse>>
```

1. Verify the authenticated user has `Action::UploadUseDownloader` privilege. This happens BEFORE the download, so unauthorized callers never trigger a network fetch.
2. Call `download::from_url(&ctx.config, body.content_url)`.
3. Return `UploadResponse` with the resulting token.

## Notes

- Files uploaded this way are temporary. Clients must not use this as a permanent storage service.
- The upload token can be referenced in other requests (such as post creation) by using `contentToken` fields in JSON bodies.
- Token-based uploads are not used within the upload API itself; tokens are consumed by other endpoints.
