# user_token.rs — User Token API Endpoints

## Purpose

This file provides HTTP API handlers for user token resources. User tokens are authentication tokens that can be used instead of passwords for API authentication.

Each token has two distinct UUIDs:
- The internal `id`, which addresses update and delete requests. It is not a secret.
- The token value, which authenticates requests. The server returns it to the client exactly once, in the create response.

The database stores only the BLAKE3 hash of the token in `token_hash` and an AES-256-GCM ciphertext in `token_secret`. The plaintext token is never stored. Token authentication hashes the presented token and looks it up by hash (see `auth/header.rs`).

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::USER_TOKEN_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::api::{self, ResourceParams, UnpagedResponse}` | Shared types and helpers |
| `crate::app::AppState` | Application state |
| `crate::auth::{header, token_crypto}` | Token hashing and at-rest encryption |
| `crate::config::Action` | Privilege actions |
| `crate::extract::{Ctx, Json, Path, Query}` | Axum extractors |
| `crate::model::enums::{AvatarStyle, ResourceOperation, ResourceType}` | Enum types |
| `crate::model::snapshot::NewSnapshot` | Snapshot creation for change tracking |
| `crate::model::user::UserToken` | Database model type |
| `crate::resource::user::MicroUser` | Simplified user info type |
| `crate::resource::user_token::{Field, UserTokenInfo, Visibility}` | Resource-layer types |
| `crate::schema::{user, user_token}` | Diesel schemas |
| `crate::snapshot` | Snapshot creation functions |
| `crate::string::{LargeString, SmallString}` | String types |
| `crate::time::DateTime` | Date-time type |
| `diesel` | Database query builder |
| `serde::Deserialize` | Deserialization |
| `serde_json::json` | JSON value construction |
| `utoipa::ToSchema` | OpenAPI schema |
| `utoipa_axum` | OpenAPI router |
| `uuid::Uuid` | UUID type for the token id and token value |

## Constants

- `MAX_EXPIRATION_OFFSET_DAYS = 3650`: Tokens may not expire more than 10 years in the future.

## Function: `validate_expiration_time`

```rust
fn validate_expiration_time(expiration_time: DateTime) -> ApiResult<DateTime>
```

Rejects expiration times that are more than 10 years in the future.

**Algorithm:**
1. Compute `max_expiration = now + 3650 days`.
2. If `expiration_time > max_expiration`, return `ApiError::FromStr("expirationTime cannot be more than 10 years in the future")`.
3. Otherwise, return the expiration time unchanged.

The `FromStr` error answers HTTP 422.

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with these route groups:
- `list` — list user tokens
- `create` — create a user token
- `update`, `delete` — modify or remove a user token

## Visibility Enum

The `Visibility` enum controls what fields are returned in the `UserTokenInfo` response:
- `Visibility::Full` — the owner sees the decrypted plaintext token
- `Visibility::PublicOnly` — the token value is hidden from non-owners

---

## Endpoint: List User Tokens

- **Method:** GET
- **Path:** `/user-tokens/{username}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenListSelf` (for own tokens) or `Action::UserTokenListAny` (for any user)

### Handler: `list`

```
async fn list(Ctx(ctx, connection_pool): Ctx, Path(username): Path<SmallString>, Query(params): Query<ResourceParams<Field>>) -> ApiResult<Json<UnpagedResponse<UserTokenInfo>>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username whose tokens to list |
| `fields` | query string | Query | Comma-separated field names |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UnpagedResponse<UserTokenInfo>` | List of user tokens |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |

### Handler Logic Step by Step

1. Look up the required privilege ranks from the config:
   - `list_any` = the rank for `Action::UserTokenListAny`
   - `list_self` = the rank for `Action::UserTokenListSelf`
2. Open a database transaction.
3. Inside the transaction:
   a. Find the user by `name = username`. Select `id` and `avatar_style`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine `is_self`: `ctx.client.id == Some(user_id)`.
   d. Select the required rank: `list_self` if viewing own tokens, `list_any` otherwise.
   e. Verify the client has at least that rank via `api::verify_privilege`.
   f. Query the `user_token` table for rows where `user_id = user_id`, ordered by `creation_time` descending.
   g. Return the tokens, avatar style, and the `is_self` flag.
4. Determine the visibility: `Visibility::Full` if `is_self` is true, otherwise `Visibility::PublicOnly`.
5. Get the encryption key from `ctx.config.token_encryption_key()`.
6. Map each `UserToken` to a `UserTokenInfo`:
   - Only the owner (full visibility) gets the token: decrypt `token_secret` with `token_crypto::decrypt_blob`.
   - On a decrypt failure, log a warning and pass `None`; the token is omitted from the response.
   - Non-owners get `None` and never see the token.
7. Collect into `UnpagedResponse.results`.
8. Return the response.

---

## Endpoint: Create User Token

- **Method:** POST
- **Path:** `/user-token/{username}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenCreateSelf` (for own tokens) or `Action::UserTokenCreateAny` (for any user)

### Handler: `create`

```
async fn create(Ctx(ctx, connection_pool): Ctx, Path(username): Path<SmallString>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<UserTokenCreateBody>) -> ApiResult<Json<UserTokenInfo>>
```

### Request Body: `UserTokenCreateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | `Option<bool>` | No | Whether the token is enabled. Defaults to `true`. |
| `note` | `Option<String>` | No | Note describing the token's purpose |
| `expirationTime` | `Option<DateTime>` | No | Optional expiration time. Defaults to 30 days from now when absent. |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username to create the token for |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UserTokenInfo` | The created user token. Contains the plaintext token value, returned only once. |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |
| 422 | `ErrorResponse` | Expiration time is more than 10 years in the future |

### Handler Logic Step by Step

1. Look up required ranks: `create_any` = `Action::UserTokenCreateAny`, `create_self` = `Action::UserTokenCreateSelf`.
2. Get the encryption key from `ctx.config.token_encryption_key()`.
3. Generate two distinct UUIDs:
   - `id = Uuid::new_v4()`: the internal identifier, stored in the `id` column and used to address later requests.
   - `token = Uuid::new_v4()`: the token value, returned to the client once.
4. Compute `token_hash = header::hash_token_hex(token.as_bytes())` and `token_secret = token_crypto::encrypt_blob(&token_key, &token.to_string())`.
5. Open a database transaction.
6. Inside the transaction:
   a. Find the user by `name = username`. Select `id` and `avatar_style`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine `is_self`. Select the required rank (`create_self` if self, `create_any` otherwise). Verify privilege.
   d. Delete any expired or disabled tokens owned by this user:
      - Filter by `user_id = user_id` AND (`enabled = false` OR `expiration_time < current_time`).
   e. Compute the expiration time:
      - If the body has no `expiration_time`, use the default: now + 30 days.
      - Validate it with `validate_expiration_time`, which rejects times more than 10 years in the future.
   f. Insert the new row with columns `id`, `user_id`, `note` (trimmed, or `""`), `enabled` (default `true`), `expiration_time`, `token_hash`, `token_secret`.
   g. Create a creation snapshot via `snapshot::user_token::creation_snapshot`.
   h. Return `(user_token, avatar_style)`.
7. Build `UserTokenInfo` with `Visibility::Full` and `Some(token.to_string())`: the creator sees the plaintext token, which is returned only this once.
8. Return the response.

---

## Endpoint: Update User Token

- **Method:** PUT
- **Path:** `/user-token/{username}/{id}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenEditSelf` (own tokens) or `Action::UserTokenEditAny` (any user)

The `{id}` path parameter is the internal user token id, not the token value.

### Handler: `update`

```
async fn update(Ctx(ctx, connection_pool): Ctx, Path((username, id)): Path<(SmallString, Uuid)>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<UserTokenUpdateBody>) -> ApiResult<Json<UserTokenInfo>>
```

### Request Body: `UserTokenUpdateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version for optimistic locking |
| `enabled` | `Option<bool>` | No | Whether the token is enabled |
| `note` | `Option<LargeString>` | No | Token description note |
| `expirationTime` | `Option<Option<DateTime>>` | No | Expiration time. An explicit `null` is rejected. Uses custom deserializer `api::deserialize_some`. |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username who owns the token |
| `id` | `Uuid` | Path | Internal user token id |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UserTokenInfo` | The updated token. The owner sees the decrypted token; others see `PublicOnly`. |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |
| 404 | `ErrorResponse` | User token does not exist |
| 409 | `ErrorResponse` | Version is outdated |
| 422 | `ErrorResponse` | Expiration time is null or more than 10 years in the future |

### Handler Logic Step by Step

1. Look up required ranks: `edit_any` and `edit_self`.
2. Get the encryption key from `ctx.config.token_encryption_key()`.
3. Open a database transaction.
4. Inside the transaction:
   a. Find the user by `name = username`. Select `id` and `avatar_style`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine `is_self`, select the required rank, verify privilege.
   d. Find the `UserToken` by `id = id` AND `user_id = user_id` (the `find(id)` primary-key lookup, filtered to the owner).
   e. If not found, return `ApiError::NotFound(ResourceType::UserToken)`.
   f. Verify the version: `user_token.last_edit_time` must match `body.version`. If not, return `ApiError::ResourceModified`.
   g. Save the current token data as `old_data` (JSON with `note`, `enabled`, `expiration_time`).
   h. Update fields if provided:
      - `body.enabled` → `user_token.enabled`
      - `body.note` → `user_token.note`
      - `body.expiration_time`:
        - `None` (field absent) → do not update.
        - `Some(None)` (explicit null) → reject with `ApiError::FromStr("expirationTime cannot be null")`; a null would permanently disable expiry.
        - `Some(Some(value))` → set the expiration, validated by `validate_expiration_time`.
   i. Set `user_token.last_edit_time` to current time.
   j. Save changes via `user_token.save_changes(conn)`.
   k. Build `new_data` JSON from the updated token.
   l. Call `snapshot::value_diff(old_data, new_data)`. If there is a difference:
      - Create a `NewSnapshot` with `ResourceOperation::Modified`, `ResourceType::UserToken`, the token id string as `resource_id`, and the diff data.
      - Insert it.
   m. Return `(updated_user_token, avatar_style, is_self)`.
5. Determine the visibility: `Visibility::Full` if the client owns the token, otherwise `Visibility::PublicOnly`.
6. Decrypt the token with `token_crypto::decrypt_blob` only for the owner. On a decrypt failure, log a warning and pass `None`.
7. Build `UserTokenInfo` with the visibility and the decrypted token.
8. Return the response.

### Note on `expiration_time` deserialization

The field uses `#[serde(default, deserialize_with = "api::deserialize_some")]`. This means:
- If the field is absent from the JSON → `None` (do not update)
- If the field is present with `null` → `Some(None)`, which the handler rejects with a 422 `FromStr` error
- If the field is present with a value → `Some(Some(value))` (set new expiration, validated against the 10-year limit)

---

## Endpoint: Delete User Token

- **Method:** DELETE
- **Path:** `/user-token/{username}/{id}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenDeleteSelf` (own tokens) or `Action::UserTokenDeleteAny` (any user)

The `{id}` path parameter is the internal user token id, not the token value.

### Handler: `delete`

```
async fn delete(Ctx(ctx, connection_pool): Ctx, Path((username, id)): Path<(String, Uuid)>) -> ApiResult<Json<()>>
```

### Request Body

None (OpenAPI declares `request_body = Object` but the handler takes no body).

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username who owns the token |
| `id` | `Uuid` | Path | Internal user token id |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | Empty JSON `()` | Token deleted |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |
| 404 | `ErrorResponse` | Token does not exist |

### Handler Logic Step by Step

1. Open a database transaction.
2. Inside the transaction:
   a. Find the user by `name = username`. Select only `id`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine the action: `Action::UserTokenDeleteSelf` if the authenticated user is the owner, `Action::UserTokenDeleteAny` otherwise.
   d. Verify privilege for the determined action.
   e. Find the `UserToken` by `id = id` AND `user_id = user_token_owner`.
   f. If not found, return `ApiError::NotFound(ResourceType::UserToken)`.
   g. Execute a `DELETE` query on the `user_token` table by token id.
   h. Create a deletion snapshot via `snapshot::user_token::deletion_snapshot`.
   i. Return `Json(())`.
3. Return the response.

---

## Tests

The test constant `FIELDS` excludes `token`, `creation_time`, `last_edit_time`, and `last_usage_time`.

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `list` | parallel | Verifies listing tokens for "administrator" |
| `list_visibility` | parallel | Verifies that the owner sees the internal id and the decrypted plaintext token, and that non-owners see the id and metadata but never the token |
| `create` | serial | Creates a token, verifies the internal id differs from the token value, verifies the stored row holds the token's BLAKE3 hash and a decryptable ciphertext, then deletes by internal id |
| `update` | serial | Edits a token by internal id and restores it, verifying the edited fields persist |
| `error` | parallel | Verifies error responses for nonexistent users, tokens, and privilege failures, plus null and far-future expiration rejection |
| `webhook_snapshots` | serial | Verifies creation and deletion snapshots, including the default expiration timestamp |
