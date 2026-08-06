# User API — `user.rs`

File: `server/src/api/user.rs`

## Overview

This module defines all user-related HTTP endpoints. It handles CRUD operations, search, avatar uploads, password changes, email changes, and rank management.

---

## Router Registration

### `routes() -> OpenApiRouter<AppState>`

**Location:** `user.rs:32`

Returns a router with these endpoints:

| Method | Path | Handler |
|--------|------|---------|
| `GET` | `/users` | `list` |
| `GET` | `/user/{name}` | `get` |
| `POST` | `/users` | `create` |
| `PUT` | `/user/{name}` | `update` |
| `DELETE` | `/user/{name}` | `delete` |

`create` and `update` use a `DefaultBodyLimit::max(config.max_multipart_body_size())` layer for the total request body; the avatar file is limited to `config.max_upload_size` by the multipart streaming code.

---

## Constants

| Name | Value | Purpose |
|------|-------|---------|
| `MAX_USERS_PER_PAGE` | `1000` | Maximum users per page in list queries |

---

## Endpoints

### 1. `GET /users` — List Users

**Handler:** `list` at `user.rs:91`

**Authentication:** Requires `Action::UserList`.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |
| `offset` | `Option<i64>` | Starting position (default: 0) |
| `limit` | `NonZeroI64` | Maximum results (capped at 1000) |

**Search Tokens (Named):**

| Token | Description |
|-------|-------------|
| `name` | Username (wildcards allowed) |
| `creation-date`, `creation-time` | Registration date |
| `last-login-date`, `last-login-time`, `login-date`, `login-time` | Last login date |

**Sort Tokens:** `random`, `name`, `creation-date`/`creation-time`, `last-login-date`/`last-login-time`/`login-date`/`login-time`

**Successful Response:** `200` with `PagedResponse<UserInfo>` (uses `Visibility::PublicOnly` for list)

**Error Responses:**
- `403` — Privileges too low

**Handler Logic:**
1. Verify `Action::UserList` privilege.
2. Cap `limit` at `MAX_USERS_PER_PAGE` (1000).
3. Open a database transaction.
4. Build `QueryBuilder` from the search query.
5. Set offset and limit.
6. Execute `query_builder.list(conn)` to get `(total, selected_user_ids)`.
7. Batch-load `UserInfo` with `Visibility::PublicOnly`.
8. Return `PagedResponse` as JSON.

---

### 2. `GET /user/{name}` — Get Single User

**Handler:** `get` at `user.rs:138`

**Authentication:** No privilege required when viewing self. Requires `Action::UserView` when viewing others.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Username |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `UserInfo`

**Error Responses:**
- `403` — Privileges too low (when viewing another user)
- `404` — User does not exist

**Handler Logic:**
1. Open a database transaction.
2. Look up the user ID by name. Return `NotFound` if absent.
3. Check if the client is viewing their own account (`ctx.client.id == Some(user_id)`).
4. If not viewing self, verify `Action::UserView` privilege.
5. Set visibility: `Visibility::Full` for self, `Visibility::PublicOnly` for others.
6. Load `UserInfo` with the appropriate visibility.
7. Return as JSON.

---

### 3. `POST /users` — Create a User

**Handler:** `create` at `user.rs:320`

**Accepts:** `application/json` or `multipart/form-data`

**Authentication:** If creating own account (not logged in), requires `Action::UserCreateSelf`. If creating another account (logged in as admin), requires `Action::UserCreateAny`.

**Request Body:** `UserCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `SmallString` | Username (must match `user_name_regex`) |
| `password` | `SmallString` | Password (must match `password_regex`) |
| `email` | `Option<SmallString>` | Email address |
| `rank` | `Option<UserRank>` | User rank; defaults to `config.default_rank()` |
| `avatarStyle` | `Option<AvatarStyle>` | Avatar style: `gravatar` or `manual` |
| `avatarToken` | `Option<UploadToken>` | Upload token for custom avatar |
| `avatarUrl` | `Option<Url>` | URL to fetch avatar from |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `UserInfo` (returns `Visibility::Full`)

**Error Responses:**
- `400` — Avatar missing for `manual` avatar style
- `403` — Privileges too low
- `403` — Trying to set rank higher than own rank
- `409` — Username already exists (`ApiError::AlreadyExists(ResourceProperty::UserName)`)
- `409` — Email already exists (`ApiError::AlreadyExists(ResourceProperty::UserEmail)`)
- `422` — Username missing/invalid (fails regex)
- `422` — Password missing/invalid (fails regex)
- `422` — Email invalid
- `422` — Rank is `Anonymous`

**Handler Logic (create_impl):**
1. If `body.rank == Some(UserRank::Anonymous)`, return `ApiError::InvalidUserRank`.
2. Determine if the client is creating their own account (`ctx.client.id.is_none()`).
3. Select action: `UserCreateSelf` or `UserCreateAny`. Verify privilege.
4. If a rank is set and it exceeds `config.default_rank()`, verify the client's privilege for that rank.
5. Validate username against `RegexType::Username`.
6. Validate password against `RegexType::Password`.
7. Validate email against `lettre::Address`.
8. Generate an Argon2 password hash with a random salt.
9. Process avatar: if `avatarStyle == Manual`, a custom avatar is required. If provided, process via `Content::thumbnail(ThumbnailType::Avatar)`.
10. Open a database transaction.
11. Check if the username already exists. If so, return `ApiError::AlreadyExists(UserName)`.
12. Get the current user count. If 0, the first user becomes `Administrator`. Otherwise, use the provided rank or `config.default_rank()`.
13. Insert `NewUser` into `user::table` with `ON CONFLICT (email) DO NOTHING`. If the row already existed, return `ApiError::AlreadyExists(UserEmail)`.
14. If a custom avatar was provided, verify the appropriate avatar privilege and call `update::user::avatar`.
15. Record a `creation_snapshot`.
16. After the first transaction, load `UserInfo` with `Visibility::Full` and return as JSON.

**Multipart Handling:**
1. Extract `PartName::Avatar` from the multipart body.
2. If metadata is missing, return `ApiError::MissingMetadata`.
3. Deserialize the metadata JSON into `UserCreateBody`.
4. If an avatar file was uploaded, set `avatar_token` on the body.

---

### 4. `PUT /user/{name}` — Update a User

**Handler:** `update` at `user.rs:538`

**Accepts:** `application/json` or `multipart/form-data`

**Authentication:** Different privileges for each field, with `Self` vs `Any` variants depending on whether the client is editing their own account.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Username |

**Request Body:** `UserUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version (required) |
| `name` | `Option<SmallString>` | New username |
| `password` | `Option<SmallString>` | New password |
| `email` | `Option<Option<SmallString>>` | New email (send `null` to clear) |
| `rank` | `Option<UserRank>` | New rank |
| `avatarStyle` | `Option<AvatarStyle>` | New avatar style |
| `avatarToken` | `Option<UploadToken>` | Upload token for new avatar |
| `avatarUrl` | `Option<Url>` | URL to fetch avatar from |

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `query` | `Option<String>` | Search query |
| `fields` | `Mask<Field>` | Field selection |

**Successful Response:** `200` with `UserInfo`

**Error Responses:**
- `400` — Avatar missing for `manual` avatar style
- `403` — Privileges too low
- `403` — Trying to set rank higher than own rank (`ApiError::InsufficientPrivileges`)
- `404` — User does not exist
- `409` — Version outdated
- `409` — New username/email already exists
- `422` — Username, password, email, or rank invalid

**Handler Logic (update_impl):**
1. If a custom avatar is provided or `avatarStyle == Some(Manual)`, process the avatar.
2. Open a database transaction.
3. Load the existing user by name. Return `NotFound` if absent.
4. Verify version.
5. Determine if editing self (`ctx.client.id == Some(user_id)`). Set visibility accordingly.
6. For each provided field:
   - **password:** If editing self, requires `UserEditSelfPass`. If editing another, requires `UserEditAnyPass`. Validate against password regex. Generate new salt and hash. Update `user::table`.
   - **email:** If editing self, requires `UserEditSelfEmail`. If editing another, requires `UserEditAnyEmail`. Validate email. Update `user::table`, mapping unique violation to `ResourceProperty::UserEmail`.
   - **rank:** If `Anonymous`, return `InvalidUserRank`. If editing self, requires `UserEditSelfRank`. If editing another, requires `UserEditAnyRank`. If new rank exceeds `config.default_rank()`, verify the client's privilege for that rank. Update `user::table`.
   - **avatar_style:** If editing self, requires `UserEditSelfAvatar`. If editing another, requires `UserEditAnyAvatar`. Update `user::table`.
   - **custom_avatar:** Same privilege verification as avatar_style. Call `update::user::avatar`.
   - **name:** If editing self, requires `UserEditSelfName`. If editing another, requires `UserEditAnyName`. Validate against username regex. Update `user::table`, mapping unique violation to `ResourceProperty::UserName`. If a custom avatar file exists at the old username path, move it to the new username path.
7. Update `last_edit_time` via `update::user::last_edit_time`.
8. Reload the user to get the final state.
9. Record a `modification_snapshot` with old and new user data.
10. Return `(user_id, visibility)`.
11. After the first transaction, load `UserInfo` and return as JSON.

**Privilege Matrix for Updates:**

| Field | Self Action | Other Action |
|-------|-------------|--------------|
| password | `UserEditSelfPass` | `UserEditAnyPass` |
| email | `UserEditSelfEmail` | `UserEditAnyEmail` |
| rank | `UserEditSelfRank` | `UserEditAnyRank` |
| avatar_style | `UserEditSelfAvatar` | `UserEditAnyAvatar` |
| avatar content | `UserEditSelfAvatar` | `UserEditAnyAvatar` |
| name | `UserEditSelfName` | `UserEditAnyName` |

---

### 5. `DELETE /user/{name}` — Delete a User

**Handler:** `delete` at `user.rs:576`

**Authentication:** If deleting self, requires `Action::UserDeleteSelf`. If deleting another, requires `Action::UserDeleteAny`.

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `String` | Username |

**Request Body:** `DeleteBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |

**Successful Response:** `200` with `()`

**Error Responses:**
- `403` — Privileges too low
- `404` — User does not exist
- `409` — Version is outdated

**Handler Logic:**
1. Open a database transaction.
2. Load the user by name. Return `NotFound` if absent.
3. Verify version.
4. Determine if deleting self. Select action: `UserDeleteSelf` or `UserDeleteAny`.
5. Verify the action privilege.
6. Delete the user from `user::table`.
7. Record a `deletion_snapshot`.
8. Return `Json(())`.

---

## Type Definitions

### `UserCreateBody`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `SmallString` | Username |
| `password` | `SmallString` | Password |
| `email` | `Option<SmallString>` | Email |
| `rank` | `Option<UserRank>` | Rank |
| `avatarStyle` | `Option<AvatarStyle>` | Avatar style |
| `avatarToken` | `Option<UploadToken>` | Avatar upload token |
| `avatarUrl` | `Option<Url>` | Avatar URL |

### `UserUpdateBody`

| Field | Type | Description |
|-------|------|-------------|
| `version` | `DateTime` | Resource version |
| `name` | `Option<SmallString>` | New username |
| `password` | `Option<SmallString>` | New password |
| `email` | `Option<Option<SmallString>>` | New email (null to clear) |
| `rank` | `Option<UserRank>` | New rank |
| `avatarStyle` | `Option<AvatarStyle>` | New avatar style |
| `avatarToken` | `Option<UploadToken>` | New avatar upload token |
| `avatarUrl` | `Option<Url>` | New avatar URL |

### `Multipart<T>`

| Field | Type | Description |
|-------|------|-------------|
| `metadata` | `T` | JSON metadata |
| `avatar` | `Option<String>` | Avatar file (binary) |
