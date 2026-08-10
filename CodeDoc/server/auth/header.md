# Authentication Header (`auth/header.rs`)

## Summary

This module handles HTTP authentication by parsing the `Authorization` header. It supports two schemes:
1. **Basic:** Base64-encoded `username:password`, verified against the password hash.
2. **Token:** Base64-encoded `username:token`, verified against the `user_token` database table.

The server never stores a plaintext token. It stores the BLAKE3 hash of the token in the `token_hash` column and an AES-256-GCM ciphertext in the `token_secret` column. Token authentication hashes the presented token and looks it up by hash. Failed basic-auth attempts are rate-limited per username with a sliding window.

## Imports

- `app::AppState`: Application state (connection pool, config).
- `auth::{self, Client}`: Authentication client type.
- `model::enums::UserRank`: User rank enum.
- `schema::{user, user_token}`: Diesel schema types.
- `time::DateTime`: Date/time type.
- `base64::prelude::BASE64_STANDARD` and `base64::{DecodeError, Engine}`: Base64 encoding.
- `diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl}`: Diesel ORM.
- `std::collections::HashMap`: Storage for the failed-login record.
- `std::str::Utf8Error`: UTF-8 conversion error.
- `std::sync::{LazyLock, Mutex}`: Static rate-limit state.
- `std::time::{Duration, Instant}`: Sliding-window timing.
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

## Enum: `AuthenticateError`

```rust
pub enum AuthenticateError {
    /// Too many failed basic-auth attempts for the username.
    RateLimited { retry_after_secs: u64 },
    /// The credentials were rejected for a regular reason.
    Auth(AuthenticationError),
}
```

Errors from [`authenticate_user`]. Rate limiting is reported separately so the middleware can answer with 429 + Retry-After.

- `RateLimited { retry_after_secs }`: The username exceeded the failure limit. `retry_after_secs` is the number of seconds to wait before the next attempt.
- `Auth(AuthenticationError)`: The credentials were rejected for a regular reason.

## Struct: `AuthenticatedClient`

```rust
pub struct AuthenticatedClient {
    pub client: Client,
    /// Hex-encoded BLAKE3 hash of the user token that authenticated this
    /// request, if the client used token-based authentication.
    pub token_hash: Option<String>,
}
```

- `client`: The authenticated client.
- `token_hash`: The hex-encoded BLAKE3 hash of the token that authenticated the request. It is `Some` only for token-based authentication. It is `None` for basic authentication. The middleware uses it to update `last_usage_time` for `?bump-login` requests.

## Function: `authenticate_user`

```rust
pub async fn authenticate_user(state: &AppState, auth: &str) -> Result<AuthenticatedClient, AuthenticateError>
```

The main entry point. Parses the `Authorization` header and authenticates the user.

**Algorithm:**
1. Split `auth` on the first space: `(auth_type, credentials)`. Return `MalformedCredentials` if no space exists.
2. Match `auth_type`:
   - `"Basic"`: Call `basic_access_authentication(state, credentials).await`.
   - `"Token"`: Call `token_authentication(state, credentials).await`.
   - Other: Return `Err(AuthenticateError::Auth(AuthenticationError::InvalidAuthType))`.

## Function: `hash_token`

```rust
pub fn hash_token(token: &[u8]) -> [u8; 32]
```

Computes the BLAKE3 hash used to store user tokens and password reset tokens. Only the hash is persisted; the plaintext token is never stored.

**Algorithm:**
1. Compute the BLAKE3 hash of `token` with `blake3::hash`.
2. Convert the digest to a 32-byte array.

## Function: `hash_token_hex`

```rust
pub fn hash_token_hex(token: &[u8]) -> String
```

Hex-encodes [`hash_token`] for storage in and lookup against the database.

**Algorithm:**
1. Call `hash_token(token)`.
2. Hex-encode the 32 bytes with `hex::encode`.

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

## Function: `basic_access_authentication(state, credentials) -> Result<AuthenticatedClient, AuthenticateError>`

Authenticates using `username:password`. Failed attempts are recorded per username and rate-limited to slow down brute-forcing; a successful login resets the username's history.

Every attempt is fully verified (DB lookup + argon2) before the rate limit is consulted, so the correct password is never rejected with 429, even during a lockout (F-1). Tradeoff: each attempt — including ones answered 429 — costs one argon2 verification; that is the same cost as before the limiter existed, and is bounded by nginx-level rate limiting.

**Algorithm:**
1. Call `decode_credentials(credentials)` to extract `(username, password)`.
2. Set `window = Duration::from_secs(config.rate_limit_window_secs)` and `now = Instant::now()`.
3. Get async database connection.
4. Query the `user` table: select `(id, rank, password_hash)` where `name = username`. Use `.optional()`.
5. Extract the stored hash. If no user found, use `auth::password::DUMMY_PASSWORD_HASH` (constant-time comparison fallback to prevent username enumeration).
6. Verify password: `auth::password::is_valid_password(&state.config, hash, &password).is_ok()`.
7. If the user was found AND the password is valid:
   - Call `clear_failed_logins(&username)`.
   - Return `AuthenticatedClient` with `Client::new(Some(user_id), rank)` and `token_hash: None`.
8. Otherwise:
   - Call `failed_login_retry_after(&username, config.rate_limit_max_failures, window, now)`.
   - If it returns `Some(retry_after_secs)`: return `Err(AuthenticateError::RateLimited { retry_after_secs })`.
   - Otherwise: call `record_failed_login(&username, window, now)` and return `Err(AuthenticationError::UsernamePasswordMismatch)`.

The limit is checked after the (failed) verification above but before recording this attempt, so the budget is unchanged from the pre-fix flow: `max_failures` failed attempts answer 401 and the next one answers 429. A correct password never reaches the failure branch, so it always succeeds (F-1).

**Security notes:**
- The dummy hash prevents a timing side channel that would reveal whether a username exists. Always runs the password verification even for unknown users.
- The rate limit never rejects the correct password, because the limit is consulted only after the password verification fails.

## Function: `token_authentication(state, credentials) -> Result<AuthenticatedClient, AuthenticateError>`

Authenticates using `username:token` where token is a UUID. The token is hashed with BLAKE3 and looked up by the stored hash.

**Algorithm:**
1. Call `decode_credentials(credentials)` to extract `(username, unparsed_token)`.
2. Parse `unparsed_token` as UUID: `Uuid::parse_str(&unparsed_token)`. On failure, return `MalformedToken`.
3. Compute `token_hash = hash_token_hex(token.as_bytes())`.
4. Get async database connection.
5. Query `user_token` table, inner-joining with `user`:
   - Select `(user::id, user::rank, user_token::enabled, user_token::expiration_time)`.
   - Filter: `user::name == username` AND `user_token::token_hash == token_hash`.
   - Use `.optional()`.
   - Return `Err(UsernameTokenMismatch)` if no row found.
6. Destructure into `(user_id, rank, enabled, expiration_time)`.
7. Check expiration: `expired = expiration_time.is_some_and(|time| time <= DateTime::now())`. A token whose expiration time equals the current time is expired.
8. If `enabled` and `!expired`: return `AuthenticatedClient` with `Client::new(Some(user_id), rank)` and `token_hash: Some(token_hash)`.
9. Otherwise: return `Err(ExpiredToken)`.

**Security note:** The plaintext token is never stored or compared. The database lookup matches on the BLAKE3 hash only, so a database leak does not expose usable tokens.

## Rate Limiting State and Helpers

### `FAILED_LOGIN_ATTEMPTS` (static)

```rust
static FAILED_LOGIN_ATTEMPTS: LazyLock<Mutex<HashMap<String, Vec<Instant>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
```

Sliding-window record of failed basic-auth attempts, keyed by the lowercased username. Names are ASCII-only per `username_regex`, so this matches the CITEXT login comparison exactly and case variants of a name share one budget (F-2). Only failed attempts count; a successful login clears the username's history.

### `MAX_FAILED_LOGIN_KEYS` (constant)

```rust
const MAX_FAILED_LOGIN_KEYS: usize = 10_000;
```

Upper bound on distinct usernames kept in `FAILED_LOGIN_ATTEMPTS`. When the map exceeds this (for example, botnet-scale username rotation), entries are pruned and then the map is cleared as a last resort, keeping memory bounded.

### `MAX_RECORDED_USERNAME_LEN` (constant)

```rust
const MAX_RECORDED_USERNAME_LEN: usize = 64;
```

Failed attempts for usernames longer than this are not recorded at all: the username regex caps name length, so such names can never match an account, cannot lock anyone out, and would only waste key space (F-3).

### Function: `failed_login_retry_after(username, max_failures, window, now) -> Option<u64>`

```rust
fn failed_login_retry_after(username: &str, max_failures: usize, window: Duration, now: Instant) -> Option<u64>
```

Returns the number of seconds to wait when `username` has exceeded `max_failures` failed attempts within `window`, or `None` when the username may still attempt authentication.

**Algorithm:**
1. Lock `FAILED_LOGIN_ATTEMPTS`.
2. Compute `cutoff = now - window`.
3. Collect the recorded timestamps for the lowercased username that are newer than `cutoff`.
4. If the count is less than `max_failures`: return `None`.
5. Otherwise: take the oldest timestamp, compute the seconds until `oldest + window`, and return it with a minimum of 1.

### Function: `record_failed_login(username, window, now)`

```rust
fn record_failed_login(username: &str, window: Duration, now: Instant)
```

Records a failed basic-auth attempt for `username`, pruning stale entries.

**Algorithm:**
1. If `username.len() > MAX_RECORDED_USERNAME_LEN`: return without recording.
2. Lock `FAILED_LOGIN_ATTEMPTS`.
3. Compute `cutoff = now - window`.
4. Get the entry for the lowercased username, or create an empty list.
5. Drop timestamps older than `cutoff`, then push `now`.
6. If the map length exceeds `MAX_FAILED_LOGIN_KEYS`:
   - Prune every key against the cutoff and drop keys that emptied out.
   - If the map still exceeds the cap (botnet-scale rotation): clear the map.

### Function: `clear_failed_logins(username)`

```rust
fn clear_failed_logins(username: &str)
```

Clears the failed-attempt history of `username` after a successful login.

**Algorithm:**
1. Lock `FAILED_LOGIN_ATTEMPTS`.
2. Remove the lowercased username key from the map.
