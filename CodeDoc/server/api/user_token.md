# user_token.rs — User Token API Endpoints

## Purpose

This file provides HTTP API handlers for user token resources. User tokens are authentication tokens that can be used instead of passwords for API authentication.

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::USER_TOKEN_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::api::{self, ResourceParams, UnpagedResponse}` | Shared types and helpers |
| `crate::app::AppState` | Application state |
| `crate::config::Action` | Privilege actions |
| `crate::extract::{Ctx, Json, Path, Query}` | Axum extractors |
| `crate::model::enums::{AvatarStyle, ResourceOperation, ResourceType}` | Enum types |
| `crate::model::snapshot::NewSnapshot` | Snapshot creation for change tracking |
| `crate::model::user::{NewUserToken, UserToken}` | Database model types |
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
| `uuid::Uuid` | UUID type for token IDs |

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with these route groups:
- `list` — list user tokens
- `create` — create a user token
- `update`, `delete` — modify or remove a user token

## Visibility Enum

The `Visibility` enum controls what fields are returned in the `UserTokenInfo` response:
- `Visibility::Full` — all fields visible (owner viewing own tokens, or right after creation)
- `Visibility::PublicOnly` — secret fields (like the raw token UUID) are hidden from non-owners

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
5. Map each `UserToken` to a `UserTokenInfo` using `UserTokenInfo::new`, passing:
   - A `MicroUser` constructed from the config, username, and avatar style
   - The token
   - The field selection
   - The visibility level
6. Collect into `UnpagedResponse.results`.
7. Return the response.

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
| `expirationTime` | `Option<DateTime>` | No | Optional expiration time |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username to create the token for |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UserTokenInfo` | The created user token (with `Visibility::Full`) |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |

### Handler Logic Step by Step

1. Look up required ranks: `create_any` = `Action::UserTokenCreateAny`, `create_self` = `Action::UserTokenCreateSelf`.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the user by `name = username`. Select `id` and `avatar_style`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine `is_self`. Select the required rank (`create_self` if self, `create_any` otherwise). Verify privilege.
   d. Delete any expired or disabled tokens owned by this user:
      - Filter by `user_id = user_id` AND (`enabled = false` OR `expiration_time < current_time`).
   e. Create a new `NewUserToken`:
      - `id`: `Uuid::new_v4()` (random UUID)
      - `user_id`: the found user's ID
      - `note`: `body.note` trimmed if `Some`, or `None`
      - `enabled`: `body.enabled.unwrap_or(true)`
      - `expiration_time`: `body.expiration_time`
   f. Insert the new token into the `user_token` table.
   g. Create a creation snapshot via `snapshot::user_token::creation_snapshot`.
   h. Return `(user_token, avatar_style)`.
4. Build `UserTokenInfo` with `Visibility::Full` (the creator sees their token UUID).
5. Return the response.

---

## Endpoint: Update User Token

- **Method:** PUT
- **Path:** `/user-token/{username}/{token}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenEditSelf` (own tokens) or `Action::UserTokenEditAny` (any user)

### Handler: `update`

```
async fn update(Ctx(ctx, connection_pool): Ctx, Path((username, token)): Path<(SmallString, Uuid)>, Query(params): Query<ResourceParams<Field>>, Json(body): Json<UserTokenUpdateBody>) -> ApiResult<Json<UserTokenInfo>>
```

### Request Body: `UserTokenUpdateBody`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | `DateTime` | Yes | Resource version for optimistic locking |
| `enabled` | `Option<bool>` | No | Whether the token is enabled |
| `note` | `Option<LargeString>` | No | Token description note |
| `expirationTime` | `Option<Option<DateTime>>` | No | Expiration time. Use null to remove expiration. Uses custom deserializer `api::deserialize_some`. |

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username who owns the token |
| `token` | `Uuid` | Path | User token UUID |
| `fields` | query string | Query | Comma-separated field names |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `UserTokenInfo` | The updated token (with `Visibility::PublicOnly`) |
| 403 | `ErrorResponse` | Insufficient privileges |
| 404 | `ErrorResponse` | User does not exist |
| 404 | `ErrorResponse` | Token does not exist |
| 409 | `ErrorResponse` | Version is outdated |

### Handler Logic Step by Step

1. Look up required ranks: `edit_any` and `edit_self`.
2. Open a database transaction.
3. Inside the transaction:
   a. Find the user by `name = username`. Select `id` and `avatar_style`.
   b. If not found, return `ApiError::NotFound(ResourceType::User)`.
   c. Determine `is_self`, select the required rank, verify privilege.
   d. Find the `UserToken` by `id = token` AND `user_id = user_id`.
   e. If not found, return `ApiError::NotFound(ResourceType::UserToken)`.
   f. Verify the version: `user_token.last_edit_time` must match `body.version`. If not, return `ApiError::ResourceModified`.
   g. Save the current token data as `old_data` (JSON with `note`, `enabled`, `expiration_time`).
   h. Update fields if provided:
      - `body.enabled` → `user_token.enabled`
      - `body.note` → `user_token.note`
      - `body.expiration_time` → `user_token.expiration_time` (note: `Option<Option<DateTime>>` — the outer `None` means "not in request", `Some(None)` means "remove expiration")
   i. Set `user_token.last_edit_time` to current time.
   j. Save changes via `user_token.save_changes(conn)`.
   k. Build `new_data` JSON from the updated token.
   l. Call `snapshot::value_diff(old_data, new_data)`. If there is a difference:
      - Create a `NewSnapshot` with `ResourceOperation::Modified`, `ResourceType::UserToken`, the token ID string as `resource_id`, and the diff data.
      - Insert it.
   m. Return `(updated_user_token, avatar_style)`.
4. Build `UserTokenInfo` with `Visibility::PublicOnly` (the raw token UUID is hidden even on updates).
5. Return the response.

### Note on `expiration_time` deserialization

The field uses `#[serde(default, deserialize_with = "api::deserialize_some")]`. This means:
- If the field is absent from the JSON → `None` (do not update)
- If the field is present with `null` → `Some(None)` (remove the expiration)
- If the field is present with a value → `Some(Some(value))` (set new expiration)

---

## Endpoint: Delete User Token

- **Method:** DELETE
- **Path:** `/user-token/{username}/{token}`
- **Authentication:** Required
- **Privilege:** `Action::UserTokenDeleteSelf` (own tokens) or `Action::UserTokenDeleteAny` (any user)

### Handler: `delete`

```
async fn delete(Ctx(ctx, connection_pool): Ctx, Path((username, token)): Path<(String, Uuid)>) -> ApiResult<Json<()>>
```

### Request Body

None (OpenAPI declares `request_body = Object` but the handler takes no body).

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `username` | `String` | Path | Username who owns the token |
| `token` | `Uuid` | Path | User token UUID |

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
   e. Find the `UserToken` by `id = token` AND `user_id = user_token_owner`.
   f. If not found, return `ApiError::NotFound(ResourceType::UserToken)`.
   g. Execute a `DELETE` query on the `user_token` table by token ID.
   h. Create a deletion snapshot via `snapshot::user_token::deletion_snapshot`.
   i. Return `Json(())`.
3. Return the response.

---

## Tests

The test constant `FIELDS` excludes `token`, `creation_time`, `last_edit_time`, and `last_usage_time`.

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `list` | parallel | Verifies listing tokens for "administrator" |
| `list_visibility` | parallel | Verifies that the owner sees raw token UUIDs but non-owners do not |
| `create` | serial | Creates a token for "restricted_user", verifies it exists, then deletes it |
| `update` | serial | Updates "administrator" token, verifies changed fields, then restores |
| `error` | parallel | Tests nonexistent user, nonexistent token, and privilege violations (regular user acting on another user) |
| `webhook_snapshots` | serial | Verifies that creating and deleting a user token generates correct snapshot data (created and deleted operations) |
