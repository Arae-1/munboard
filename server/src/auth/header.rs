use crate::app::AppState;
use crate::auth::{self, Client};
use crate::model::enums::UserRank;
use crate::schema::{user, user_token};
use crate::time::DateTime;
use base64::prelude::BASE64_STANDARD;
use base64::{DecodeError, Engine};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use std::collections::HashMap;
use std::str::Utf8Error;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

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

/// Errors from [`authenticate_user`]. Rate limiting is reported separately so
/// the middleware can answer with 429 + Retry-After.
pub enum AuthenticateError {
    /// Too many failed basic-auth attempts for the username.
    RateLimited { retry_after_secs: u64 },
    /// The credentials were rejected for a regular reason.
    Auth(AuthenticationError),
}

/// Outcome of a successful authentication.
pub struct AuthenticatedClient {
    pub client: Client,
    /// Hex-encoded BLAKE3 hash of the user token that authenticated this
    /// request, if the client used token-based authentication.
    pub token_hash: Option<String>,
}

/// Authentication can either be done by token-based authentication (recommended)
/// or by sending password as plaintext.
pub async fn authenticate_user(state: &AppState, auth: &str) -> Result<AuthenticatedClient, AuthenticateError> {
    let (auth_type, credentials) = auth
        .split_once(' ')
        .ok_or(AuthenticateError::Auth(AuthenticationError::MalformedCredentials))?;
    match auth_type {
        "Basic" => basic_access_authentication(state, credentials).await,
        "Token" => token_authentication(state, credentials).await,
        _ => Err(AuthenticateError::Auth(AuthenticationError::InvalidAuthType)),
    }
}

/// Computes the BLAKE3 hash used to store user tokens and password reset tokens.
/// Only the hash is persisted; the plaintext token is never stored.
pub fn hash_token(token: &[u8]) -> [u8; 32] {
    blake3::hash(token).into()
}

/// Hex-encodes [`hash_token`] for storage in and lookup against the database.
pub fn hash_token_hex(token: &[u8]) -> String {
    hex::encode(hash_token(token))
}

#[cfg(test)]
pub fn basic_credentials_for(username: &str, password: &str) -> String {
    let credentials = format!("{username}:{password}");
    format!("Basic {}", BASE64_STANDARD.encode(credentials))
}

#[cfg(test)]
pub fn token_credentials_for(username: &str, token: Uuid) -> String {
    let credentials = format!("{username}:{token}");
    format!("Token {}", BASE64_STANDARD.encode(credentials))
}

/// `credentials` are sent base64 encoded, so this function decodes them to utf-8.
fn decode_credentials(credentials: &str) -> Result<(String, String), AuthenticationError> {
    let decoded_credentials = BASE64_STANDARD.decode(credentials)?;
    let utf8_encoded_credentials = std::str::from_utf8(&decoded_credentials)?;
    utf8_encoded_credentials
        .split_once(':')
        .map(|(username, pass)| (username.to_owned(), pass.to_owned()))
        .ok_or(AuthenticationError::MalformedCredentials)
}

/// Checks that the given `credentials` are of the form "username:password"
/// and that the username/password combination is valid.
///
/// Failed attempts are recorded per username and rate-limited to slow down
/// brute-forcing; a successful login resets the username's history.
///
/// Every attempt is fully verified (DB lookup + argon2) before the rate limit
/// is consulted, so the correct password is never rejected with 429, even
/// during a lockout (F-1). Tradeoff: each attempt — including ones answered
/// 429 — costs one argon2 verification; that is the same cost as before the
/// limiter existed, and is bounded by nginx-level rate limiting.
async fn basic_access_authentication(
    state: &AppState,
    credentials: &str,
) -> Result<AuthenticatedClient, AuthenticateError> {
    let (username, password) = decode_credentials(credentials).map_err(|err| AuthenticateError::Auth(err.into()))?;

    let window = Duration::from_secs(state.config.rate_limit_window_secs);
    let now = Instant::now();

    let mut conn = state.connection_pool.get().await.map_err(|err| AuthenticateError::Auth(err.into()))?;

    // For security reasons, don't give any indication to the user if it was the password
    // or the username that was incorrect.
    let result: Option<(i64, UserRank, String)> = user::table
        .select((user::id, user::rank, user::password_hash))
        .filter(user::name.eq(&username))
        .first(conn.as_mut())
        .optional()
        .map_err(|err| AuthenticateError::Auth(err.into()))?;
    let hash = result
        .as_ref()
        .map(|(_, _, h)| h.as_str())
        .unwrap_or(auth::password::DUMMY_PASSWORD_HASH);
    let password_ok = auth::password::is_valid_password(&state.config, hash, &password).is_ok();
    match result.filter(|_| password_ok) {
        Some((user_id, rank, _)) => {
            clear_failed_logins(&username);
            Ok(AuthenticatedClient {
                client: Client::new(Some(user_id), rank),
                token_hash: None,
            })
        }
        None => {
            // The limit is checked after the (failed) verification above but
            // before recording this attempt, so the budget is unchanged from
            // the pre-fix flow: `max_failures` failed attempts answer 401 and
            // the next one answers 429. A correct password never reaches this
            // branch, so it always succeeds (F-1).
            if let Some(retry_after_secs) =
                failed_login_retry_after(&username, state.config.rate_limit_max_failures, window, now)
            {
                return Err(AuthenticateError::RateLimited { retry_after_secs });
            }
            record_failed_login(&username, window, Instant::now());
            Err(AuthenticateError::Auth(AuthenticationError::UsernamePasswordMismatch))
        }
    }
}

/// Checks that the given `credentials` are of the form "username:token"
/// and that the username/token combination is valid and non-expired.
async fn token_authentication(state: &AppState, credentials: &str) -> Result<AuthenticatedClient, AuthenticateError> {
    let (username, unparsed_token) = decode_credentials(credentials).map_err(|err| AuthenticateError::Auth(err.into()))?;
    let token = Uuid::parse_str(&unparsed_token).map_err(|err| AuthenticateError::Auth(err.into()))?;
    let token_hash = hash_token_hex(token.as_bytes());
    let mut conn = state.connection_pool.get().await.map_err(|err| AuthenticateError::Auth(err.into()))?;

    let (user_id, rank, enabled, expiration_time): (i64, UserRank, bool, Option<DateTime>) = user_token::table
        .inner_join(user::table)
        .select((user::id, user::rank, user_token::enabled, user_token::expiration_time))
        .filter(user::name.eq(username))
        .filter(user_token::token_hash.eq(token_hash.clone()))
        .first(conn.as_mut())
        .optional()
        .map_err(|err| AuthenticateError::Auth(err.into()))?
        .ok_or(AuthenticateError::Auth(AuthenticationError::UsernameTokenMismatch))?;

    let expired = expiration_time.as_ref().is_some_and(|&time| time <= DateTime::now());
    (enabled && !expired)
        .then_some(AuthenticatedClient {
            client: Client::new(Some(user_id), rank),
            token_hash: Some(token_hash),
        })
        .ok_or(AuthenticateError::Auth(AuthenticationError::ExpiredToken))
}

/// Sliding-window record of failed basic-auth attempts, keyed by the
/// lowercased username (names are ASCII-only per `username_regex`, so this
/// matches the CITEXT login comparison exactly and case variants of a name
/// share one budget — F-2). Only failed attempts count; a successful login
/// clears the username's history.
static FAILED_LOGIN_ATTEMPTS: LazyLock<Mutex<HashMap<String, Vec<Instant>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Upper bound on distinct usernames kept in [`FAILED_LOGIN_ATTEMPTS`]. When
/// the map exceeds this (e.g. botnet-scale username rotation), entries are
/// pruned and then the map is cleared as a last resort, keeping memory bounded.
const MAX_FAILED_LOGIN_KEYS: usize = 10_000;

/// Failed attempts for usernames longer than this are not recorded at all:
/// the username regex caps name length, so such names can never match an
/// account, cannot lock anyone out, and would only waste key space (F-3).
const MAX_RECORDED_USERNAME_LEN: usize = 64;

/// Returns the number of seconds to wait when `username` has exceeded
/// `max_failures` failed attempts within `window`, or `None` when the username
/// may still attempt authentication.
fn failed_login_retry_after(username: &str, max_failures: usize, window: Duration, now: Instant) -> Option<u64> {
    let attempts = FAILED_LOGIN_ATTEMPTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let cutoff = now.checked_sub(window).unwrap_or(now);
    let failures: Vec<Instant> = attempts
        .get(&username.to_lowercase())
        .map(|times| times.iter().copied().filter(|&time| time > cutoff).collect())
        .unwrap_or_default();
    (failures.len() >= max_failures).then(|| {
        let oldest = failures.iter().min().copied().unwrap_or(now);
        (oldest + window).saturating_duration_since(now).as_secs().max(1)
    })
}

/// Records a failed basic-auth attempt for `username`, pruning stale entries.
fn record_failed_login(username: &str, window: Duration, now: Instant) {
    if username.len() > MAX_RECORDED_USERNAME_LEN {
        return;
    }
    let mut attempts = FAILED_LOGIN_ATTEMPTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let cutoff = now.checked_sub(window).unwrap_or(now);
    let failures = attempts.entry(username.to_lowercase()).or_default();
    failures.retain(|&time| time > cutoff);
    failures.push(now);

    if attempts.len() > MAX_FAILED_LOGIN_KEYS {
        // Prune every key against the cutoff and drop keys that emptied out.
        attempts.retain(|_, times| {
            times.retain(|&time| time > cutoff);
            !times.is_empty()
        });
        // Still over the cap (botnet-scale rotation): start over rather than
        // letting the map grow without bound.
        if attempts.len() > MAX_FAILED_LOGIN_KEYS {
            attempts.clear();
        }
    }
}

/// Clears the failed-attempt history of `username` after a successful login.
fn clear_failed_logins(username: &str) {
    let mut attempts = FAILED_LOGIN_ATTEMPTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    attempts.remove(&username.to_lowercase());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn failed_logins_are_rate_limited_within_the_window() {
        const USERNAME: &str = "rate_limited_user";
        const MAX_FAILURES: usize = 2;

        clear_failed_logins(USERNAME);
        let window = Duration::from_secs(1);
        let start = Instant::now();

        // Failures below the limit are allowed.
        assert!(failed_login_retry_after(USERNAME, MAX_FAILURES, window, start).is_none());
        record_failed_login(USERNAME, window, start + Duration::from_millis(100));
        assert!(failed_login_retry_after(USERNAME, MAX_FAILURES, window, start + Duration::from_millis(200)).is_none());

        // Reaching the limit denies further attempts and reports a retry delay.
        record_failed_login(USERNAME, window, start + Duration::from_millis(300));
        let retry_after = failed_login_retry_after(USERNAME, MAX_FAILURES, window, start + Duration::from_millis(400));
        assert!(retry_after.is_some_and(|secs| secs >= 1));

        // Once the window has passed, the username is allowed again.
        assert!(failed_login_retry_after(USERNAME, MAX_FAILURES, window, start + Duration::from_secs(2)).is_none());

        // A successful login clears the failure history.
        record_failed_login(USERNAME, window, start + Duration::from_secs(3));
        clear_failed_logins(USERNAME);
        assert!(failed_login_retry_after(USERNAME, MAX_FAILURES, window, start + Duration::from_secs(4)).is_none());
    }
}
