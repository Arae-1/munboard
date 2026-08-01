# API Router — `mod.rs`

File: `server/src/api/mod.rs`

## Overview

This file defines the top-level API router structure. It merges all sub-routers into one `OpenApiRouter`. It also defines shared types and helper functions that the endpoint modules use.

---

## Router Construction

### `routes(state: AppState) -> OpenApiRouter`

This function builds the full API router. It merges 15 sub-routers in this order:

| Order | Sub-router module   | Source file         |
|-------|----------------------|----------------------|
| 1     | `comment`            | `comment.rs`         |
| 2     | `custom_field`       | `custom_field.rs`    |
| 3     | `info`               | `info.rs`            |
| 4     | `legacy`             | `legacy.rs`          |
| 5     | `password_reset`     | `password_reset.rs`  |
| 6     | `pool`               | `pool.rs`            |
| 7     | `pool_category`      | `pool_category.rs`   |
| 8     | `post`               | `post.rs`            |
| 9     | `snapshot`           | `snapshot.rs`        |
| 10    | `tag`                | `tag.rs`             |
| 11    | `tag_category`       | `tag_category.rs`    |
| 12    | `upload`             | `upload.rs`          |
| 13    | `user`               | `user.rs`            |
| 14    | `user_token`         | `user_token.rs`      |

The OpenAPI spec is initialized from `ApiDoc::openapi()`.

### Middleware Layers

The router applies these layers in order (outermost first):

| Layer | Description |
|-------|-------------|
| `TraceLayer::new_for_http()` | HTTP request tracing |
| `TimeoutLayer::with_status_code(408, 1 min)` | Request timeout of 1 minute |
| `middleware::auth` | Authentication and authorization |
| `middleware::post_to_webhooks` | Webhook posting after request |
| `middleware::log_error` | Error logging |

The router also sets:
- A fallback handler that returns `404 Route not found` for unmatched paths.
- `AppState` as the shared state, available to all handlers.

---

## Shared Type Definitions

### `RatingBody`

**Location:** `mod.rs:87`

Request body to apply or change a score.

| Field | Type   | Description |
|-------|--------|-------------|
| `score` | `Rating` | The rating score to apply |

Implements `Deref<Target = Rating>`.

### `DeleteBody`

**Location:** `mod.rs:99`

Request body for deleting a resource.

| Field | Type      | Description |
|-------|-----------|-------------|
| `version` | `DateTime` | Resource version for optimistic concurrency control |

Implements `Deref<Target = DateTime>`.

### `MergeBody<T>`

**Location:** `mod.rs:113`

Request body for merging two resources.

| Field | Type | Description |
|-------|------|-------------|
| `remove` | `T` | ID of the source resource to be removed |
| `merge_to` | `T` | ID of the target resource to merge into |
| `remove_version` | `DateTime` | Version of the source resource |
| `merge_to_version` | `DateTime` | Version of the target resource |

Uses `#[serde(rename_all = "camelCase")]`.

### `ResourceParams<F>`

**Location:** `mod.rs:127`

Query parameters for retrieving one or more resources.

| Field | Type | Description |
|-------|------|-------------|
| `query` | `Option<String>` | Search query string |
| `fields` | `Mask<F>` | Comma-separated field selection |

The `criteria()` method returns the query string or an empty string if `None`.

### `PageParams`

**Location:** `mod.rs:144`

Pagination query parameters for listing resources.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `offset` | `Option<i64>` | `None` (treated as 0) | Starting position |
| `limit` | `NonZeroI64` | (required) | Maximum results to return |

### `PagedResponse<T>`

**Location:** `mod.rs:162`

A paged search result.

| Field | Type | Description |
|-------|------|-------------|
| `query` | `Option<String>` | The original search query |
| `offset` | `i64` | The starting offset |
| `limit` | `i64` | Records per page |
| `total` | `i64` | Total matching resources |
| `results` | `Vec<T>` | The result array |

### `UnpagedResponse<T>`

**Location:** `mod.rs:157`

A search result without paging.

| Field | Type | Description |
|-------|------|-------------|
| `results` | `Vec<T>` | The result array |

---

## Helper Functions

### `verify_privilege(client, required_rank) -> ApiResult<()>`

**Location:** `mod.rs:181`

Checks if `client.rank >= required_rank`. Returns `ApiError::InsufficientPrivileges` if the check fails.

### `verify_version(current_version, client_version) -> ApiResult<()>`

**Location:** `mod.rs:189`

Checks if `current_version == client_version`. Returns `ApiError::ResourceModified` if they differ.
This check is **disabled** in test builds (`cfg!(test)`).

### `verify_matches_regex(config, haystack, regex_type) -> ApiResult<()>`

**Location:** `mod.rs:68`

Checks if `haystack` matches the configured regex of type `regex_type`. Returns `ApiError::ExpressionFailsRegex` if the check fails.

### `verify_valid_email(email) -> Result<()>`

**Location:** `mod.rs:78`

Parses an optional email string with `lettre::Address`. Returns `Ok` if `None` or if it is a valid address. Returns `lettre::address::AddressError` on failure.

### `deserialize_some<'de, T, D>(deserializer) -> Result<Option<T>>`

**Location:** `mod.rs:201`

A custom serde deserializer. Any present value (including `null`) becomes `Some(value)`. Only absent fields become `None`. This allows clients to send `null` to clear fields.

---

## Submodule Declarations

All sub-routers are declared as private modules at `mod.rs:21-37`:

```rust
mod comment;
mod custom_field;
mod doc;
pub mod error;
mod info;
mod legacy;
pub mod middleware;
mod password_reset;
mod pool;
mod pool_category;
mod post;
mod snapshot;
mod tag;
mod tag_category;
mod upload;
mod user;
mod user_token;
```

`error` and `middleware` are `pub`. All others are `mod` (private to the API crate).

---

## Imports

The file imports types from:
- `crate::api::doc::ApiDoc` — the OpenAPI document definition
- `crate::api::error` — error types `ApiError` and `ApiResult`
- `crate::app::AppState` — application state
- `crate::auth::Client` — authenticated client
- `crate::config` — configuration and `RegexType`
- `crate::model::enums` — `Rating` and `UserRank`
- `crate::resource::field::Mask` — field selection mask
- `crate::string::SmallString` — compact string type
- `crate::time::DateTime` — timestamp type
- `axum::http::StatusCode` — HTTP status codes
- `tower_http` — timeout and trace layers
- `utoipa` — OpenAPI schema annotations
- `utoipa_axum::router::OpenApiRouter` — the router builder
