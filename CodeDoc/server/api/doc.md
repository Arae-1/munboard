# doc.rs — OpenAPI Documentation Definitions

## Purpose

This file defines the OpenAPI documentation constants, tags, and descriptions for the entire API. It does not contain any handlers. It defines the `ApiDoc` struct used by `utoipa` to generate the OpenAPI schema.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::error::ErrorName` | Error name enum for schema generation |
| `utoipa::OpenApi` | Derive macro for the OpenAPI document |

## OpenAPI Tag Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `COMMENT_TAG` | `"Comment"` | Comment API tag |
| `CUSTOM_FIELD_TAG` | `"Custom-Field"` | Custom Field API tag |
| `INFO_TAG` | `"Info"` | Info API tag |
| `PASSWORD_RESET_TAG` | `"Password-Reset"` | Password Reset API tag |
| `POOL_TAG` | `"Pool"` | Pool API tag |
| `POOL_CATEGORY_TAG` | `"Pool-Category"` | Pool Category API tag |
| `POST_TAG` | `"Post"` | Post API tag |
| `SNAPSHOT_TAG` | `"Snapshot"` | Snapshot API tag |
| `TAG_TAG` | `"Tag"` | Tag API tag |
| `TAG_CATEGORY_TAG` | `"Tag-Category"` | Tag Category API tag |
| `UPLOAD_TAG` | `"Upload"` | Upload API tag |
| `USER_TAG` | `"User"` | User API tag |
| `USER_TOKEN_TAG` | `"User-Token"` | User Token API tag |

Each constant is used in the corresponding API endpoint file to group routes under the same tag in the OpenAPI documentation.

---

## `ApiDoc` Struct

```
#[derive(OpenApi)]
#[openapi(
    components(schemas(ErrorName)),
    tags(...)
)]
pub struct ApiDoc;
```

This struct is the root OpenAPI document. It includes:
- `ErrorName` in the component schemas
- All tag definitions with descriptions

---

## Tag Definitions

| Tag Name | Description |
|----------|-------------|
| `Comment` | Comment API endpoints |
| `Custom-Field` | Custom Field API endpoints |
| `Info` | Info API endpoints |
| `Password-Reset` | Password reset API endpoints |
| `Pool` | Pool API endpoints |
| `Pool-Category` | Pool category API endpoints |
| `Post` | Post API endpoints |
| `Snapshot` | Snapshot API endpoints |
| `Tag` | Tag API endpoints |
| `Tag-Category` | Tag category API endpoints |
| `Upload` | Upload API endpoints |
| `User` | User API endpoints |
| `User-Token` | User token API endpoints |
| `Authentication` | Documentation about HTTP Basic authentication |
| `User-Token-Authentication` | Documentation about token-based authentication |
| `Basic-Requests` | Documentation about request format requirements |
| `File-Uploads` | Documentation about file upload encoding |
| `Field-Selection` | Documentation about field selection query parameter |
| `Versioning` | Documentation about optimistic locking |
| `Webhooks` | Documentation about webhook notifications |
| `Search` | Documentation about search query syntax |
| `Errors` | Documentation about error response format |

---

## Documentation Text Constants

### `AUTHENTICATION_DESCRIPTION`

Describes:
- HTTP Basic authentication mechanism
- Recommendation to use HTTPS
- No sessions — every privileged request must be authenticated
- Privileges depend on user rank as defined in server configuration
- Recommendation to add `?bump-login` query parameter to the first request of a client session to update last login time

### `USER_TOKEN_AUTHENTICATION_DESCRIPTION`

Describes:
- Token authentication similar to Basic auth
- Uses `Authorization: Token <base64(username:token)>` header format
- Still recommends HTTPS
- Benefits: no need to transmit password repeatedly, tokens can be revoked individually

### `BASIC_REQUESTS_DESCRIPTION`

States that all requests must use:
- `Content-Type: application/json`
- `Accept: application/json`
- Except for file upload requests

### `FILE_UPLOADS_DESCRIPTION`

Describes three upload methods:
1. **Multipart form data**: Send files as multipart parts with a `metadata` part containing JSON
2. **URL download**: Pass URLs in JSON fields with `Url` suffix (e.g., `contentUrl`)
3. **Token reuse**: Use previously uploaded files by referencing tokens with `Token` suffix (e.g., `contentToken`)

Also mentions `yt-dlp` integration for content download from sites like YouTube (requires `uploads:use_downloader` permission).

### `FIELD_SELECTION_DESCRIPTION`

Describes the `?fields=field1,field2,...` query parameter that clients can use to select specific top-level fields in responses. Works on all request types (GET, PUT, etc.).

Example: `GET /posts/?fields=id,tags`

### `VERSIONING_DESCRIPTION`

Describes optimistic locking:
- Each modifiable resource has a `version` field returned in GET responses
- PUT and DELETE requests must include the same `version` value
- If the version does not match (resource was modified by someone else), the server rejects the request

The version value is a timestamp in ISO 8601 format.

### `WEBHOOKS_DESCRIPTION`

Describes webhook functionality:
- System administrators configure webhook URLs in `config.toml`
- On any event, the server sends a POST request to configured URLs
- The request body contains a snapshot resource generated with anonymous user privileges
- Content format is JSON

### `SEARCH_DESCRIPTION`

Describes the search query syntax:
- **Token types**: anonymous, named (`key:value`), sort style (`sort:style`), special (`special:value`)
- **Ranged/composite values**: comma-separated lists (`a,b,c`), ranges (`1..`, `..4`, `1..4`)
- **Date/time values**: `today`, `yesterday`, `<year>`, `<year>-<month>`, `<year>-<month>-<day>`
- **Wildcards**: fields that accept names (like user names) accept `*`
- **Negation**: prepend `-` to negate any token
- **Sort direction**: append `,asc` or `,desc` to sort tokens
- **Escaping**: use backslash before special characters (`:`, `*`, `,`)

Example query: `sea -fav-count:8.. type:png uploader:Pirate,Davy`

### `ERROR_DESCRIPTION`

Describes the error response format:
- All errors return appropriate HTTP status code and JSON body
- JSON body structure: `{ "name": "...", "title": "...", "description": "..." }`
- `name`: specific error name (e.g., `PostNotFound`)
- `title`: generic category (e.g., `Resource Not Found`)
- `description`: detailed message
- Lists possible error names in the `ErrorName` schema
