# Authentication Header (`auth/header.rs`)

## Summary

This module handles HTTP authentication by parsing the `Authorization` header. It supports two schemes:
1. **Basic:** Base64-encoded `username:password`, verified against the password hash.
2. **Token:** Base64-encoded `username:token`, verified against the `user_token` database table.

## Imports

- `app::AppState`: Application state (connection pool, config).
- `auth::{self, Client}`: Authentication client type.
- `model::enums::UserRank`: User rank enum.
- `schema::{user, user_token}`: Diesel schema types.
- `time::DateTime`: Date/time type.
- `base64::prelude::BASE64_STANDARD` and `base64::{DecodeError, Engine}`: Base64 encoding.
- `diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl}`: Diesel ORM.
- `std::str::Utf8Error`: UTF-8 conversion error.
- `thiserror::Error`: Error derive.
- `uuid::Uuid`: UUID type.

## Enum: `AuthenticationError`

```rust
#[derive(Debug, Error)]
#[error(transparent)]
pub enum AuthenticationError {
    #[error("Token has expired")]
    ExpiredToken,
    FailedConnection(#[from] diesel::r2d2::PoolError),
    FailedQuery(#[from] diesel::result::Error),
    #[error("Invalid authentication type")]
    InvalidAuthType,
    InvalidEncoding(#[from] DecodeError),
    #[error("Authentication credentials are malformed")]
    MalformedCredentials,
    MalformedToken(#[from] uuid::Error),
    #[error("Invalid username and password combination")]
    UsernamePasswordMismatch,
    #[error("Invalid username and token combination")]
    UsernameTokenMismatch,
    Utf8Conversion(#[from] Utf8Error),
}
```

Error variants for authentication failures. Uses `#[error(transparent)]` for source chain. The `#[from]` derives auto-convert from specific error types.

## Function: `authenticate_user(state, auth) -> Result<Client, AuthenticationError>`

```rust
pub async fn authenticate_user(state: &AppState, auth: &str) -> Result<Client, AuthenticationError>
```

The main entry point. Parses the `Authorization` header and authenticates the user.

**Algorithm:**
1. Split `auth` on the first space: `(auth_type, credentials)`. Return `MalformedCredentials` if no space exists.
2. Match `auth_type`:
   - `"Basic"`: Call `basic_access_authentication(state, credentials).await`.
   - `"Token"`: Call `token_authentication(state, credentials).await`.
   - Other: Return `Err(AuthenticationError::InvalidAuthType)`.

## Function: `basic_credentials_for(username, password) -> String` (test only)

```rust
#[cfg(test)]
pub fn basic_credentials_for(username: &str, password: &str) -> String
```

Constructs a Basic authentication header value for testing: `"Basic {base64(username:password)}"`.

## Function: `token_credentials_for(username, token) -> String` (test only)

```rust
#[cfg(test)]
pub fn token_credentials_for(username: &str, token: Uuid) -> String
```

Constructs a Token authentication header value for testing: `"Token {base64(username:token)}"`.

## Function: `decode_credentials(credentials: &str) -> Result<(String, String), AuthenticationError>`

Decodes Base64-encoded credentials into a `(username, password_or_token)` pair.

**Algorithm:**
1. Decode Base64: `BASE64_STANDARD.decode(credentials)`.
2. Convert bytes to UTF-8 string: `std::str::from_utf8(&decoded_credentials)`.
3. Split on `:`: `split_once(':')`.
4. If a split exists: return `(username.to_owned(), password_or_token.to_owned())`.
5. If no split: return `Err(MalformedCredentials)`.

## Function: `basic_access_authentication(state, credentials) -> Result<Client, AuthenticationError>`

Authenticates using `username:password`.

**Algorithm:**
1. Call `decode_credentials(credentials)` to extract `(username, password)`.
2. Get async database connection.
3. Query the `user` table: select `(id, rank, password_hash)` where `name = username`. Use `.optional()`.
4. Extract the stored hash. If no user found, use `auth::password::DUMMY_PASSWORD_HASH` (constant-time comparison fallback to prevent username enumeration).
5. Verify password: `auth::password::is_valid_password(&state.config, hash, &password)`.
6. If the hash check passes AND the user was found: return `Client::new(Some(user_id), rank)`.
7. Otherwise: return `Err(UsernamePasswordMismatch)`.

**Security note:** The dummy hash prevents a timing side channel that would reveal whether a username exists. Always runs the password verification even for unknown users.

## Function: `token_authentication(state, credentials) -> Result<Client, AuthenticationError>`

Authenticates using `username:token` where token is a UUID.

**Algorithm:**
1. Call `decode_credentials(credentials)` to extract `(username, unparsed_token)`.
2. Parse `unparsed_token` as UUID: `Uuid::parse_str(&unparsed_token)`.
3. Get async database connection.
4. Query `user_token` table, inner-joining with `user`:
   - Select `(user::id, user::rank, user_token::enabled, user_token::expiration_time)`.
   - Filter: `user::name == username` AND `user_token::id == token`.
   - Use `.optional()`.
   - Return `Err(UsernameTokenMismatch)` if no row found.
5. Destructure into `(user_id, rank, enabled, expiration_time)`.
6. Check expiration: `expired = expiration_time.is_some_and(|time| time < DateTime::now())`.
7. If `enabled` and `!expired`: return `Client::new(Some(user_id), rank)`.
8. Otherwise: return `Err(ExpiredToken)`.
