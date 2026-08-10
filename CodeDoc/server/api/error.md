# error.rs — API Error Types

## Purpose

This file defines the `ApiError` enum, the `ApiResult` type alias, error-to-HTTP-status mapping, database error mapping helpers, and the `ErrorResponse` JSON body type.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::auth::header::AuthenticationError` | Authentication error type |
| `crate::config::RegexType` | Regex type enum for error context |
| `crate::error::{ErrorKind, ErrorName}` | Error trait and name enum |
| `crate::model::enums::{MimeType, ResourceProperty, ResourceType}` | Enum types for resource context |
| `crate::string::SmallString` | Compact string |
| `axum::Json` | JSON response body |
| `axum::http::StatusCode` | HTTP status codes |
| `axum::response::{IntoResponse, Response}` | Axum response conversion |
| `diesel::QueryResult` | Database query result type |
| `image::error::{ImageError, LimitError, LimitErrorKind}` | Image processing errors |
| `serde::Serialize` | Serialization |
| `std::sync::Arc` | Atomic reference counting for error extension |
| `thiserror::Error` | Derive macro for error types |
| `utoipa::ToSchema` | OpenAPI schema |

## Type Alias

### `ApiResult<T>`

```
pub type ApiResult<T> = Result<T, ApiError>;
```

The standard return type for all API handler functions.

---

## Struct: `ByteSize`

```
pub struct ByteSize(pub usize);
```

A wrapper around a byte count. Provides a human-readable `Display` format for size limits.

The `Display` implementation:
1. Starts with the byte count and the unit `B`.
2. While the value is at least 1024 and the unit is not `PiB`, divides the value by 1024 and moves to the next unit.
3. Writes the value without decimals if the division is exact.
4. Writes the value with two decimals otherwise.

The unit names are `B`, `KiB`, `MiB`, `GiB`, `TiB`, `PiB`. For example, `ByteSize(4294967296)` displays as `4 GiB`.

---

## Error Enum: `ApiError`

An exhaustive enum covering all possible API error conditions. Each variant has a `#[error("...")]` attribute for display formatting, and multiple variants use `#[from]` to enable automatic conversion via the `?` operator.

### Error Variants

| Variant | Display Message | Source |
|---------|----------------|--------|
| `AlreadyExists(ResourceProperty)` | `"{0} already exists"` | Direct |
| `BlockedUrl` | `"URL targets a prohibited address"` | Direct |
| `ContentTypeMismatch(MimeType, SmallString)` | `"File of type {0} did not match request with content-type '{1}'"` | Direct |
| `CyclicDependency(ResourceType)` | `"Cyclic dependency detected in {0}s"` | Direct |
| `DeleteDefault(ResourceType)` | `"Cannot delete default {0}"` | Direct |
| `FileTooLarge(ByteSize)` | `"File exceeds maximum size of {0}"` (`{0}` displays as a human-readable size) | Direct |
| `MetadataTooLarge(ByteSize)` | `"Metadata exceeds maximum size of {0}"` (`{0}` displays as a human-readable size) | Direct |
| `EmptySwf` | `"SWF has no decodable images"` | Direct |
| `EmptyVideo` | `"Video file has no frames"` | Direct |
| `ExpressionFailsRegex(SmallString, RegexType)` | `"'{0}' does not match on {1} regex"` | Direct |
| `ExtensionRejection` | (delegates to inner) | `axum::extract::rejection::ExtensionRejection` |
| `FailedAuthentication` | (delegates to inner) | `AuthenticationError` |
| `FailedConnection` | (delegates to inner) | `diesel::r2d2::PoolError` |
| `FailedEmailTransport` | (delegates to inner) | `lettre::transport::smtp::Error` |
| `FailedQuery` | (delegates to inner) | `diesel::result::Error` |
| `FfmpegError(Box<dyn Error>)` | (delegates to inner) | Direct |
| `FrameBufferMismatch(u32, u32, usize)` | `"Image buffer of length {2} does not match dimensions {0}x{1}"` | Direct |
| `FromStr` | (delegates to inner) | `Box<dyn Error>` |
| `HeaderDeserialization` | (delegates to inner) | `axum::http::header::ToStrError` |
| `Hidden(ResourceType)` | `"{0} hidden"` | Direct |
| `InsufficientPrivileges` | `"Insufficient privileges"` | Direct |
| `InvalidMagicBytes(MimeType)` | `"Stated MIME type {0} does not match file magic bytes"` | Direct |
| `InvalidEmailAddress` | (delegates to inner) | `lettre::address::AddressError` |
| `InvalidEmail` | (delegates to inner) | `lettre::error::Error` |
| `InvalidHeader` | (delegates to inner) | `reqwest::header::InvalidHeaderValue` |
| `InvalidSort` | `"Invalid sort token"` | Direct |
| `InvalidTime` | (delegates to inner) | `crate::search::TimeParsingError` |
| `InvalidUploadToken` | `"Upload token is invalid"` | Direct |
| `InvalidUserRank` | `"Cannot create an anonymous user"` | Direct |
| `Image` | (delegates to inner) | `image::ImageError` |
| `JsonRejection` | (delegates to inner) | `axum::extract::rejection::JsonRejection` |
| `JsonSerialization` | (delegates to inner) | `serde_json::Error` |
| `MissingContent(ResourceType)` | `"Missing {0} content"` | Direct |
| `MissingContentType` | `"Form is missing content-type"` | Direct |
| `MissingFormData` | `"Missing form data"` | Direct |
| `MissingMetadata` | `"Missing metadata form"` | Direct |
| `MissingSmtpInfo` | `"Missing smtp info"` | Direct |
| `Multipart` | (delegates to inner) | `axum::extract::multipart::MultipartError` |
| `MultipartRejection` | (delegates to inner) | `axum::extract::multipart::MultipartRejection` |
| `NoNamesGiven(ResourceType)` | `"{0} needs at least one name"` | Direct |
| `NotAnInteger` | (delegates to inner) | `std::num::ParseIntError` |
| `NotFound(ResourceType)` | `"{0} not found"` | Direct |
| `NotLoggedIn` | `"This action requires you to be logged in"` | Direct |
| `Password` | (delegates to inner) | `argon2::password_hash::Error` |
| `PathRejection` | (delegates to inner) | `axum::extract::rejection::PathRejection` |
| `QueryRejection` | (delegates to inner) | `axum::extract::rejection::QueryRejection` |
| `QueryTooLong(ByteSize)` | `"Search query exceeds maximum length of {0}"` (`{0}` displays as a human-readable size) | Direct |
| `Request` | (delegates to inner) | `reqwest::Error` |
| `ResourceModified` | `"Someone else modified this in the meantime. Please try again."` | Direct |
| `SelfMerge(ResourceType)` | `"Cannot merge {0} with itself"` | Direct |
| `StdIo` | (delegates to inner) | `std::io::Error` |
| `SwfDecoding` | (delegates to inner) | `swf::error::Error` |
| `TaskJoin` | (delegates to inner) | `tokio::task::JoinError` |
| `UnauthorizedPasswordReset` | `"Password reset token is invalid"` | Direct |
| `UnsupportedExtension` | (delegates to inner) | `crate::model::enums::ParseExtensionError` |

---

## HTTP Status Code Mapping

### Method: `status_code(&self) -> StatusCode`

| Status Code | Error Variants |
|-------------|---------------|
| **400 BAD REQUEST** | `ContentTypeMismatch`, `HeaderDeserialization`, `InvalidMagicBytes`, `MissingContent`, `MissingContentType`, `MissingFormData`, `MissingMetadata`, `QueryTooLong`, plus `JsonSerialization` with `Syntax`/`Data` category |
| **401 UNAUTHORIZED** | `NotLoggedIn`, `Password`, `UnauthorizedPasswordReset`, plus `FailedAuthentication` non-connection/non-query variants |
| **403 FORBIDDEN** | `Hidden`, `InsufficientPrivileges` |
| **404 NOT FOUND** | `NotFound` |
| **409 CONFLICT** | `AlreadyExists`, `ResourceModified` |
| **415 UNSUPPORTED MEDIA TYPE** | `UnsupportedExtension` |
| **422 UNPROCESSABLE ENTITY** | `BlockedUrl`, `CyclicDependency`, `DeleteDefault`, `EmptySwf`, `EmptyVideo`, `ExpressionFailsRegex`, `FfmpegError`, `FileTooLarge`, `FrameBufferMismatch`, `FromStr`, `Image`, `InvalidEmail`, `InvalidEmailAddress`, `InvalidSort`, `InvalidTime`, `InvalidUploadToken`, `InvalidUserRank`, `MetadataTooLarge`, `NoNamesGiven`, `NotAnInteger`, `SelfMerge`, `SwfDecoding` |
| **500 INTERNAL SERVER ERROR** | `FailedEmailTransport`, `FailedQuery`, `InvalidHeader`, `MissingSmtpInfo`, `StdIo`, `TaskJoin`, plus `JsonSerialization` with `Io`/`Eof` category, plus `FailedAuthentication::FailedQuery` |
| **503 SERVICE UNAVAILABLE** | `FailedConnection`, `FailedAuthentication::FailedConnection` |

Errors that wrap Axum rejection types (`ExtensionRejection`, `JsonRejection`, `MultipartRejection`, `PathRejection`, `QueryRejection`) delegate to the inner rejection's `status()` method. `Request` (reqwest error) uses `err.status().unwrap_or(StatusCode::BAD_REQUEST)`.

---

## Error Category

### Method: `category(&self) -> &'static str`

Returns a human-readable category string for each error variant. Examples:
- `AlreadyExists(_)` → `"Already Exists"`
- `NotFound(_)` → `"Resource Not Found"`
- `InsufficientPrivileges` → `"Insufficient Privileges"`
- `ResourceModified` → `"Resource Modified"`
- `FileTooLarge(_)` → `"File Too Large"`
- `MetadataTooLarge(_)` → `"Metadata Too Large"`
- `QueryTooLong(_)` → `"Query Too Long"`

Every variant has a corresponding category string.

---

## Error Response Format

### Method: `response(&self) -> ErrorResponse`

Returns an `ErrorResponse` with three fields:
- `name`: The error name (from `self.kind()` which returns an `ErrorName` enum variant)
- `title`: The category string (e.g., `"Resource Not Found"`)
- `description`: The description string. The value depends on the error variant:
  - For `Multipart` errors, uses `err.body_text()`.
  - For internal errors, uses `"An internal error occurred."`.
  - For all other errors, uses the `Display` message (e.g., `"User not found"`).

### Method: `is_internal_error(&self) -> bool`

Returns `true` for unexpected server-side failures whose `Display` may embed raw internal detail, for example database or connection error text. Such detail must not be sent to clients.

The masked variants are:
- `FailedConnection`
- `FailedEmailTransport`
- `FailedQuery`
- `InvalidHeader`
- `StdIo`
- `TaskJoin`
- `FailedAuthentication` with `FailedConnection` or `FailedQuery`

The full detail is still logged server-side by the `log_error` middleware, which records `%err.to_string()` for every response.

### JSON Body: `ErrorResponse`

```json
{
    "name": "ErrorName",
    "title": "Error Category",
    "description": "Detailed error description"
}
```

---

## IntoResponse Implementation

```
impl IntoResponse for ApiError
```

1. Get the HTTP status code via `self.status_code()`.
2. Build the `ErrorResponse` JSON via `self.response()`.
3. Create the HTTP response tuple `(status_code, Json(error_response))`.
4. Insert `Arc<ApiError>` into the response extensions so the `log_error` middleware can access it.
5. Return the response.

---

## Database Error Mapping Functions

### `map_unique_violation<T>`

```
pub fn map_unique_violation<T>(result: QueryResult<T>, property: ResourceProperty) -> ApiResult<T>
```

Maps a `QueryResult` error:
- If the error is a `DatabaseErrorKind::UniqueViolation`, returns `ApiError::AlreadyExists(property)`.
- Otherwise, converts the diesel error via `From`.

### `map_foreign_key_violation<T>`

```
pub fn map_foreign_key_violation<T>(result: QueryResult<T>, resource: ResourceType) -> ApiResult<T>
```

Maps a `QueryResult` error:
- If the error is a `DatabaseErrorKind::ForeignKeyViolation`, returns `ApiError::NotFound(resource)`.
- Otherwise, converts via `From`.

### `map_unique_or_foreign_key_violation<T>`

```
pub fn map_unique_or_foreign_key_violation<T>(result: QueryResult<T>, unique_property: ResourceProperty, foreign_resource: ResourceType) -> ApiResult<T>
```

Handles both violation types:
- `UniqueViolation` → `ApiError::AlreadyExists(unique_property)`
- `ForeignKeyViolation` → `ApiError::NotFound(foreign_resource)`
- Other errors → converted via `From`

---

## From Implementation: `LimitErrorKind`

```
impl From<LimitErrorKind> for ApiError
```

Converts image `LimitErrorKind` to `ApiError::Image(ImageError::Limits(LimitError::from(value)))`. This enables use of the `?` operator on image operations that produce `LimitErrorKind`.
