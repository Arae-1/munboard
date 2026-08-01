# password_reset.rs — Password Reset API Endpoints

## Purpose

This file provides HTTP API handlers for the password reset flow. The flow has two steps: requesting a reset (which sends an email with a token) and confirming the reset with the token (which generates a new temporary password).

## Module Imports

| Import | Purpose |
|--------|---------|
| `crate::api::doc::PASSWORD_RESET_TAG` | OpenAPI documentation tag |
| `crate::api::error::{ApiError, ApiResult}` | Error types |
| `crate::app::AppState` | Application state |
| `crate::auth::password` | Password hashing functions |
| `crate::content::hash` | URL-safe hash computation |
| `crate::extract::{Json, Path}` | Axum extractors |
| `crate::schema::user` | User table schema |
| `crate::string::SmallString` | Compact string type |
| `crate::time::DateTime` | Date-time type |
| `argon2::password_hash::{SaltString, rand_core::{OsRng, RngCore}}` | Password hashing and randomness |
| `axum::extract::State` | AppState extractor |
| `diesel` | Database query builder |
| `lettre` | Email message building and SMTP transport |
| `percent_encoding::NON_ALPHANUMERIC` | URL percent-encoding |
| `serde::{Deserialize, Serialize}` | JSON serialization |
| `std::str::FromStr` | String parsing |
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
| 200 | Empty JSON `()` | Request processed (always, even if user does not exist, for security) |

### Handler Logic Step by Step

1. Extract SMTP information from `state.config.smtp()`. If SMTP is not configured (returns `None`), return `ApiError::MissingSmtpInfo`.
2. Get a database connection from the pool.
3. Query the `user` table for a user matching either the `name` or `email` equal to `identifier`.
   - Select columns: `id`, `name`, `email`, `password_reset_token`, `password_reset_expires`.
4. If no user is found, return `Ok(Json(()))` immediately. This prevents user enumeration.
5. If the user has no email configured (`user_email` is `None`), return `Ok(Json(()))`.
6. If both `password_reset_token` and `password_reset_expires` exist and the expiry time is in the future, return `ApiError::ResetAlreadyRequested`.
7. Parse the user's email into a `lettre::Address`. If parsing fails, return `Ok(Json(()))`.
8. Construct the mailing `Mailbox` from the address.
9. Determine the domain for the reset link URL:
   a. First try `state.config.domain`.
   b. If not set, try the `HTTP_ORIGIN` environment variable.
   c. If not set, try the `HTTP_REFERER` environment variable.
   d. If not set, try the `PORT` environment variable to build `http://localhost:{port}`.
   e. If none of these work, use an empty string.
10. Trim any trailing `/` from the domain.
11. Get the site name from `state.config.public_info.name`.
12. Percent-encode the username and a colon separator for the URL.
13. Generate 32 random bytes using `OsRng`.
14. Compute a URL-safe hash of those bytes via `hash::compute_url_safe_hash`. This becomes the `reset_token`.
15. Calculate the expiry time: 1 hour from now.
16. Build the reset URL: `{domain}/password-reset/{username_encoded}{separator}{reset_token}`.
17. Build an HTML email message with the reset link:
    - From: `smtp_info.from`
    - To: `user_mailbox`
    - Subject: `Password reset for {site_name}`
    - Body: Contains the reset link in an HTML anchor tag.
18. If the email message fails to build, return `Ok(Json(()))`.
19. Update the user's `password_reset_token` and `password_reset_expires` in the database.
20. Build an SMTP transport:
    a. Relay through `smtp_info.host`.
    b. If username and password are configured, add SMTP credentials.
    c. If port is configured, set the port.
21. Send the email via the SMTP transport. Ignore the result (the `_` binding).
22. Return `Ok(Json(()))`.

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
| `identifier` | `String` | Path | User email or username |

### Request Body: `ResetToken`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `token` | `String` | Yes | The reset token from the email link |

### Response

| Status | Body | Description |
|--------|------|-------------|
| 200 | `NewPassword` | New temporary password as plain text |
| 400 | `ErrorResponse` | Token is missing or invalid |
| 404 | `ErrorResponse` | User does not exist |

### Response Schema: `NewPassword`

| Field | Type | Description |
|-------|------|-------------|
| `password` | `String` | The new temporary password. 16 printable ASCII characters. |

### Handler Logic Step by Step

1. Open a database transaction with the connection pool.
2. Inside the transaction:
   a. Query the `user` table filtered by `name = username`.
   b. Select columns: `id`, `name`, `email`, `password_reset_token`, `password_reset_expires`.
   c. If the user does not exist, return `ApiError::UnauthorizedPasswordReset`.
   d. Extract `stored_token`. If it is `None`, return `ApiError::UnauthorizedPasswordReset`.
   e. Extract `expires`. If it is `None`, return `ApiError::UnauthorizedPasswordReset`.
   f. Compare `confirmation.token` with `stored_token`. If not equal, return `ApiError::UnauthorizedPasswordReset`.
   g. Check if the current time is past the `expires` time. If so, return `ApiError::UnauthorizedPasswordReset`.
   h. Clear the user's `password_reset_token` and `password_reset_expires` by setting them to `None`.
   i. Generate a temporary password of 16 printable ASCII characters using `generate_temporary_password(16)`.
   j. Generate a new `SaltString`.
   k. Hash the temporary password with the salt using `password::hash_password`.
   l. Update the user's `password_salt` and `password_hash` in the database.
   m. Return `Json(NewPassword { password: temporary_password })`.
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
