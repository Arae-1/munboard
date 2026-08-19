# password_reset.rs — Password Reset API Endpoints

## Purpose

This file provides HTTP API handlers for the password reset flow. The flow has two steps: requesting a reset (which sends an email with a token) and confirming the reset with the token (which generates a new temporary password).

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::PASSWORD_RESET_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::app::AppState` | Application state |
| `crate::auth::{header, password}` | Token hashing (`hash_token_hex`) and password hashing |
| `crate::content::hash` | URL-safe hash computation |
| `crate::extract::{Json, Path}` | Axum extractors |
| `crate::schema::{user, user_token}` | User and user token table schemas |
| `crate::string::SmallString` | Compact string type |
| `crate::time::DateTime` | Date-time type |
| `argon2::password_hash::{SaltString, rand_core::{OsRng, RngCore}}` | Password hashing and randomness |
| `axum::extract::State` | AppState extractor |
| `diesel` | Database query builder |
| `lettre` | Email message building and SMTP transport |
| `percent_encoding::NON_ALPHANUMERIC` | URL percent-encoding |
| `serde::{Deserialize, Serialize}` | JSON serialization |
| `std::str::FromStr` | String parsing |
| `subtle::ConstantTimeEq` | Constant-time comparison of token hashes |
| `utoipa::ToSchema` | OpenAPI schema |
| `utoipa_axum` | OpenAPI router |

## Router

### `routes() -> OpenApiRouter<AppState>`

Returns a router with two routes: `request_reset` and `reset_password`.

---

## Endpoint: Request Password Reset

- **Method:** GET
- **Path:** `/password-reset/{identifier}`
- **Authentication:** None

### Handler: `request_reset`

```
async fn request_reset(State(state): State<AppState>, Path(identifier): Path<SmallString>) -> ApiResult<Json<()>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `identifier` | `SmallString` | Path | User email or username |

### Request Body

None.

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | Empty JSON `()` | Request processed (always, for security) |

The response is an immediate, identical 200. The code returns this status regardless of whether the user exists, has an email configured, or already has a pending reset token. This prevents user enumeration.

### Handler Logic Step by Step

1. Check `state.config.smtp()`. If SMTP is not configured, return `ApiError::MissingSmtpInfo`. This check is config-level and identical for every identifier, so it cannot be used to enumerate users.
2. Spawn a background task with `tokio::spawn`. The task calls `send_reset_email(state, identifier)`.
3. If the background task fails, log the error at `warn` level. The error never surfaces in the response.
4. Return `Ok(Json(()))` immediately.

Every identifier-dependent step (user lookup, token generation, the database write and the email send) happens in the background task.

---

## Background Task: `send_reset_email`

```
async fn send_reset_email(state: AppState, identifier: SmallString) -> ApiResult<()>
```

This function performs the actual password-reset request work: it looks up the user, builds the confirmation email, conditionally reserves a fresh reset token, and sends the email.

A still-valid pending token is never overwritten and no duplicate email is sent. The no-overwrite guard and token insert share one transaction. PostgreSQL rechecks the WHERE clause of a waiting UPDATE against the committed row. Only the first insert matches; the second skips the email. SMTP and task failures are logged and do not surface in the response.

### Handler Logic Step by Step

1. Extract SMTP information from `state.config.smtp()`. If SMTP is not configured, return `ApiError::MissingSmtpInfo`.
2. Get a database connection from the pool.
3. Query the `user` table for a user matching either the `name` or `email` equal to `identifier`.
   - Select columns: `id`, `name`, `email`, `password_reset_token`, `password_reset_expires`.
4. If no user is found, return `Ok(())`.
5. If the user has no email configured (`email` is `None`), return `Ok(())`.
6. Parse the user's email into a `lettre::Address`. If parsing fails, return `Ok(())`.
7. Construct the mailing `Mailbox` from the address.
8. Determine the domain for the reset link URL:
   a. First try `state.config.domain`.
   b. If not set, try the `HTTP_ORIGIN` environment variable.
   c. If not set, try the `HTTP_REFERER` environment variable.
   d. If not set, try the `PORT` environment variable to build `http://localhost:{port}`.
   e. If none of these work, use an empty string.
9. Trim any trailing `/` from the domain.
10. Generate 32 random bytes using `OsRng`.
11. Compute a URL-safe hash of those bytes via `hash::compute_url_safe_hash`. This becomes the `reset_token`.
12. Calculate the expiry time: 1 hour from now.
13. Store only the hash of the token with `header::hash_token_hex(reset_token.as_bytes())`. The database never stores the token itself, so a database leak cannot yield live reset links.
14. Build the reset URL: `{domain}/password-reset/{username_encoded}{separator}{reset_token}`. The username and a colon separator are percent-encoded.
15. Build an HTML email message with the reset link:
    - From: `smtp_info.from`
    - To: `user_mailbox`
    - Subject: `Password reset for {site_name}`
    - Body: Contains the reset link in an HTML anchor tag.
16. If the email message fails to build, return `Ok(())` without reserving a token.
17. Run a conditional UPDATE in one transaction. The UPDATE targets the user row where `id = user_id` and the pending token is absent or expired:
    - Condition: `password_reset_token IS NULL` OR `password_reset_expires < now`.
    - Set: `password_reset_token` = stored token hash, `password_reset_expires` = expiry time.
18. If the UPDATE matched 0 rows, a still-valid pending token already exists. Do not overwrite it and do not send another email. Return `Ok(())`.
19. Send the email in an awaited blocking task (`tokio::task::spawn_blocking`):
    a. Build an SMTP transport by relaying through `smtp_info.host`.
    b. If username and password are configured, add SMTP credentials.
    c. If a port is configured, set the port.
    d. Send the email via the SMTP transport.
20. If SMTP transport construction, sending, or blocking-task execution fails, log the failure at `warn` level. Conditionally clear `password_reset_token` and `password_reset_expires` only when both still equal the hash and expiry reserved by this request, so a newer token is never cleared.
21. Return `Ok(())`.

The email is built before the token is stored. A failed delivery clears this request's reservation, allowing a later request to retry, while a still-valid token remains protected from replacement.

---

## Endpoint: Reset Password (Confirm)

- **Method:** POST
- **Path:** `/password-reset/{identifier}`
- **Authentication:** None

### Handler: `reset_password`

```
async fn reset_password(State(state): State<AppState>, Path(username): Path<SmallString>, Json(confirmation): Json<ResetToken>) -> ApiResult<Json<NewPassword>>
```

### Request Parameters

| Parameter | Type | Source | Description |
|-----------|------|--------|-------------|
| `identifier` | `String` | Path | Username |

The confirm endpoint resolves the user by name only. This differs from `request_reset`, which also accepts an email address.

### Request Body: `ResetToken`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `token` | `String` | Yes | The reset token from the email link |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `NewPassword` | New temporary password as plain text |
| 401 | `ErrorResponse` | `UnauthorizedPasswordReset`: user does not exist, token is missing or invalid, token is expired, or the submitted hash does not match |

Every failure inside the transaction returns `ApiError::UnauthorizedPasswordReset`, which maps to status `401`.

### Response Schema: `NewPassword`

| Field | Type | Description |
|-------|------|-------------|
| `password` | `String` | The new temporary password. 16 printable ASCII characters. |

### Handler Logic Step by Step

1. Open a database transaction with the connection pool.
2. Inside the transaction:
   a. Query the `user` table filtered by `name = username`.
   b. Select columns: `id`, `name`, `email`, `password_reset_token`, `password_reset_expires`.
   c. Lock the user row with `FOR UPDATE`. This closes the token-reuse TOCTOU: a concurrent confirm with the same token blocks on this read, re-reads the committed row after the first one commits, sees the cleared token, and fails with `UnauthorizedPasswordReset`. Both confirms cannot succeed.
   d. If the user does not exist, return `ApiError::UnauthorizedPasswordReset`.
   e. Extract `stored_token`. If it is `None`, return `ApiError::UnauthorizedPasswordReset`.
   f. Extract `expires`. If it is `None`, return `ApiError::UnauthorizedPasswordReset`.
   g. Compute `header::hash_token_hex(confirmation.token.as_bytes())`.
   h. Compare the submitted hash with the stored hash in constant time using `subtle::ConstantTimeEq`. If they differ, return `ApiError::UnauthorizedPasswordReset`.
   i. Check if the current time is past the `expires` time. If so, return `ApiError::UnauthorizedPasswordReset`.
   j. Clear the user's `password_reset_token` and `password_reset_expires` by setting them to `None`.
   k. Generate a temporary password of 16 printable ASCII characters using `generate_temporary_password(16)`.
   l. Generate a new `SaltString`.
   m. Hash the temporary password with the salt using `password::hash_password`.
   n. Update the user's `password_salt` and `password_hash` in the database.
   o. Revoke all of the user's API tokens: delete every `user_token` row with the user's `user_id`. A stolen token does not survive a password reset.
   p. Return `Json(NewPassword { password: temporary_password })`.
3. Return the response.

### Helper Function: `generate_temporary_password`

```
fn generate_temporary_password(length: u8) -> String
```

Generates a random string of printable ASCII characters from `'!'` (0x21) to `'~'` (0x7E). The number of possible characters is 94 (`b'~' - b'!'`). For each position, `OsRng` generates a random `u32`, maps it to a character in the printable ASCII range, and collects them into a `String`.

## Tests

| Test Function | Serial | Description |
|---------------|--------|-------------|
| `error` | parallel | Tests requesting reset for nonexistent user, confirming for nonexistent user, no email case, invalid email case, reset disabled (no SMTP), invalid token, and missing token (no token body) |
| `reset_password_revokes_api_tokens` | serial | Seeds a pending reset token and an API token, confirms the reset through the API, and asserts the API token is revoked |
| `request_reset_cleans_token_after_smtp_failure_for_username_and_email` | serial | Points SMTP at a failing port, asserts the immediate 200 with a null body, and polls the database until failed background sends clear their reservations for both username and email lookup; also verifies a valid token is not replaced |
