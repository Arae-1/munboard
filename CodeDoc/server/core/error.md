# error.rs — Error Kind Mapping

## Purpose

This file defines `ErrorName`, an exhaustive enum of all named error types in the system, and `ErrorKind`, a trait that maps library and application errors to `ErrorName` values. This provides structured, machine-readable error classification for API responses and OpenAPI documentation.

## Enum: `ErrorName`

A flat enum with 218 variants representing every distinct error condition in the system. Variants derive `Serialize` and `ToSchema`.

**Categories of error names:**

| Category | Example Variants |
|----------|-----------------|
| Network/IO | `AddressInUse`, `ConnectionRefused`, `FileNotFound`, `PermissionDenied`, `TimedOut` |
| Database | `CheckViolation`, `UniqueViolation`, `ForeignKeyViolation`, `Deadlock`, `RowNotFound` |
| Authentication | `ExpiredToken`, `InsufficientPrivileges`, `NotLoggedIn`, `MalformedCredentials` |
| Resource not found | `CommentNotFound`, `PostNotFound`, `TagNotFound`, `UserNotFound`, `PoolNotFound` |
| Resource conflict | `UserNameAlreadyExists`, `TagNameAlreadyExists`, `DuplicatePost`, `SelfMerge` |
| Validation | `ExpressionFailsRegex`, `InvalidPassword`, `ValueTooLong`, `ValueTooShort` |
| Content processing | `FFmpegError`, `GenericImageError`, `UnsupportedFormat`, `EmptyVideo` |
| Email | `EmailMissingTo`, `EmailAddressInvalidDomain` |
| JSON parsing | `JsonSyntaxError`, `JsonUnexpectedEOF` |
| SWF | `SwfParseError`, `SwfUnsupported` |
| General | `DeserializationError`, `SerializationError`, `RequestError` |

## Trait: `ErrorKind`

```rust
pub trait ErrorKind {
    fn kind(&self) -> ErrorName;
}
```

Converts any error type to its corresponding `ErrorName`.

## Trait Implementations (all `impl ErrorKind for ...`)

| Type | Source | Mapping Logic |
|------|--------|---------------|
| `argon2::B64Error` | Password hashing errors | `InvalidEncoding`, `InvalidLength` |
| `argon2::InvalidValue` | Password hash validation | `InvalidCharacter`, `InvalidFormat`, `MalformedValue`, `ValueTooLong`, `ValueTooShort`, fallback: `UnknownArgonInvalidValue` |
| `argon2::password_hash::Error` | All password hash errors | Maps each variant; `B64Encoding` and `ParamValueInvalid` delegate to inner error's `kind()` |
| `axum::MultipartRejection` | Multipart form rejections | `InvalidBoundary`, fallback: `UnknownMultipartRejectionError` |
| `axum::path::ErrorKind` | Path parameter errors | `WrongNumberOfPathParameters`, `ParseErrorAtKey`, `ParseErrorAtIndex`, `ParseError`, `InvalidUtf8InPathParam`, `UnsupportedPathType`, `PathDeserializeError`, `OtherPathError` |
| `axum::JsonRejection` | JSON body rejections | `JsonDataError`, `JsonSyntaxError`, `MissingJsonContentType`, `BytesRejection` |
| `axum::PathRejection` | Path extractor rejections | Delegates to inner error; `MissingPathParams` |
| `axum::QueryRejection` | Query string rejections | `FailedToDeserializeQueryString` |
| `base64::DecodeError` | Base64 errors | `InvalidByte`, `InvalidLastSymbol`, `InvalidLength`, `InvalidPadding` |
| `crate::auth::header::AuthenticationError` | Auth header errors | `ExpiredToken`, `FailedConnection`, `FailedQuery` (delegates), `InvalidAuthType`, `InvalidEncoding` (delegates), `MalformedCredentials`, `MalformedToken`, `UsernamePasswordMismatch`, `UsernameTokenMismatch`, `Utf8Conversion` |
| `crate::model::enums::ResourceProperty` | Resource uniqueness conflicts | Maps each property to its "AlreadyExists" variant |
| `crate::model::enums::ResourceType` | Resource not found | Maps each type to its "NotFound" variant |
| `crate::search::TimeParsingError` | Time query parsing | `TooFewArgs`, `TooManyArgs`, `NotAnInteger` (delegates), `OutOfRange` |
| `diesel::DatabaseErrorKind` | Database constraint errors | `CheckViolation`, `ClosedConnection`, `ForeignKeyViolation`, `NotNullViolation`, `ReadOnlyTransaction`, `SerializationFailure`, `UnableToSendCommand`, `UniqueViolation` |
| `diesel::result::Error` | All diesel errors | Maps each variant; delegates to inner error for `DatabaseError` and `RollbackErrorOnCommit` |
| `diesel::ConnectionError` | Connection errors | `BadConnection`, `CouldntSetupConfiguration` (delegates), `InvalidCString`, `InvalidConnectionUrl` |
| `image::LimitErrorKind` | Image dimension/limit errors | `DimensionLimitsExceeded`, `InsufficientMemory`, `UnsupportedImageDimensions` |
| `image::ParameterErrorKind` | Image parameter errors | `DimensionMismatch`, `FailedAlready`, `GenericImageError`, `NoMoreData` |
| `image::UnsupportedErrorKind` | Unsupported image format/color | `UnsupportedColor`, `UnsupportedFormat`, `UnsupportedFeature` |
| `image::ImageError` | All image errors | Maps each variant, delegates `IoError`, `Limits`, `Parameter`, `Unsupported` to inner `kind().kind()` |
| `lettre::AddressError` | Email address errors | `MissingParts`, `Unbalanced`, `InvalidUser`, `InvalidDomain`, `InvalidInput` |
| `lettre::error::Error` | Email transport errors | `MissingFrom`, `MissingTo`, `TooManyFrom`, `EmailMissingAt`, `EmailMissingLocalPart`, `EmailMissingDomain`, `CannotParseFilename`, `Io` (delegates), `NonAsciiChars` |
| `serde_json::Category` | JSON error categories | `JsonIoError`, `JsonInvalidSyntax`, `JsonInvalidData`, `JsonUnexpectedEOF` |
| `std::env::VarError` | Environment variable errors | `NotPresent`, `NotUnicode` |
| `std::io::ErrorKind` | I/O errors | Exhaustive mapping of all standard I/O error kinds |
| `std::num::IntErrorKind` | Integer parse errors | `EmptyValue`, `InvalidDigit`, `PosOverflow`, `NegOverflow`, `ZeroNotAllowed` |
| `swf::error::Error` | SWF decoding errors | `Avm1ParseError`, `InvalidData`, `SwfParseError`, `IoError`, `Unsupported` |
| `tokio::task::JoinError` | Async task errors | `TaskPanicked` if panic, `TaskCancelled` otherwise |
| `crate::api::error::ApiError` | Application-level errors | Exhaustive mapping; delegates to inner error `kind()` where applicable |

## Pattern

The double `.kind().kind()` pattern (e.g., `err.kind().kind()`) appears when an outer error wraps an inner error that also implements `ErrorKind`. The first `kind()` extracts the inner error type, the second maps it to `ErrorName`.
