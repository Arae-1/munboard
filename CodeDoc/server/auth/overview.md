# Authentication System

## Individual Files

- [header.md](header.md)
- [password.md](password.md)
- [token_crypto.md](token_crypto.md)

## Purpose

The auth module provides client identity verification. It supports two authentication methods: Basic (username + password) and Token (username + token). It also provides Argon2id password hashing and AES-256-GCM at-rest encryption of user tokens. The server stores only a BLAKE3 hash and an encrypted blob for each token; it never stores the plaintext token.

## Client (`mod.rs`)

### `Client`
A struct representing an authenticated client:
- `id: Option<i64>`: The user's ID, or None if anonymous.
- `rank: UserRank`: The user's privilege rank.

### `new(id, rank)`
Constructs a Client from an optional user ID and a rank.

## Header Authentication (`header.rs`)

### `AuthenticationError`
Error types:
- `ExpiredToken`: The token has passed its expiration time.
- `FailedConnection`: Database pool error.
- `FailedQuery`: Database query error.
- `InvalidAuthType`: The authentication type is not "Basic" or "Token".
- `InvalidEncoding`: Base64 decoding failed.
- `MalformedCredentials`: The credentials format is invalid.
- `MalformedToken`: The UUID token format is invalid.
- `UsernamePasswordMismatch`: The username and password do not match.
- `UsernameTokenMismatch`: The username and token do not match.
- `Utf8Conversion`: UTF-8 decoding of credentials failed.

### `AuthenticateError`
Errors from `authenticate_user`, reported separately so the middleware can answer with 429 + Retry-After:
- `RateLimited { retry_after_secs }`: Too many failed basic-auth attempts for the username.
- `Auth(AuthenticationError)`: The credentials were rejected for a regular reason.

### `AuthenticatedClient`
Outcome of a successful authentication:
- `client: Client`: The authenticated client.
- `token_hash: Option<String>`: The hex-encoded BLAKE3 hash of the token, present only for token-based authentication.

### `authenticate_user`
The main entry point. Returns `Result<AuthenticatedClient, AuthenticateError>`. Parses the Authorization header value:
1. Splits on the first space to get the auth type and credentials.
2. Routes to `basic_access_authentication` for "Basic" or `token_authentication` for "Token".
3. Returns `InvalidAuthType` for unknown types.

### Basic Access Authentication
1. Decodes the base64-encoded `username:password` pair.
2. Queries the database for the user's ID, rank, and password hash.
3. If the user does not exist, uses a dummy password hash to prevent timing attacks.
4. Verifies the password against the stored hash using Argon2id.
5. On success, clears the failed-attempt history and returns the client.
6. On failure, checks a per-username sliding-window rate limit. Over the limit, it returns `RateLimited`; otherwise it records the attempt and returns `UsernamePasswordMismatch`.

The rate limit uses `rate_limit_max_failures` (default 20) failures within `rate_limit_window_secs` (default 60) seconds. Keys are lowercased usernames, so case variants of a name share one budget. The limit is checked only after a failed verification, so the correct password is never rejected with 429.

### Token Authentication
1. Decodes the base64-encoded `username:token` pair.
2. Parses the token as a UUID.
3. Computes the BLAKE3 hash of the token with `hash_token_hex`.
4. Joins `user_token` with `user` on `token_hash` and the username to get the user ID, rank, token enabled status, and expiration time.
5. Checks that the token is enabled and not expired. A token is expired when `expiration_time <= now`.
6. Returns the client and the token hash on success, `UsernameTokenMismatch` or `ExpiredToken` on failure.

The plaintext token is never stored or compared; the database lookup matches on the BLAKE3 hash only.

### Helper Functions
- `decode_credentials`: Base64 decodes the credentials string and splits on `:`.
- `hash_token`: Computes the BLAKE3 hash of a token, returning 32 bytes.
- `hash_token_hex`: Hex-encodes `hash_token` for storage in and lookup against the database.
- `basic_credentials_for`: Test helper that creates a "Basic" auth header value.
- `token_credentials_for`: Test helper that creates a "Token" auth header value.

## Token Encryption (`token_crypto.rs`)

Encrypts and decrypts user tokens at rest with AES-256-GCM:
- `parse_key`: Parses the 64-character hex config key into 32 bytes.
- `encrypt` / `decrypt`: Encrypt and decrypt with a separate 12-byte nonce.
- `encrypt_blob`: Returns one stored blob: 12-byte nonce followed by ciphertext.
- `decrypt_blob`: Decrypts a blob produced by `encrypt_blob`.
- `TokenCryptoError`: Error variants `Decrypt`, `Utf8`, and `ShortBlob`.

The key comes from the `token_encryption_secret` config value. The API layer encrypts tokens on create and decrypts them for the owner on list and update.

## Password Hashing (`password.rs`)

### `DUMMY_PASSWORD_HASH`
A hardcoded Argon2id hash string used as a fallback when a user is not found, to prevent timing-based username enumeration.

### `hash_password`
Hashes a plaintext password using Argon2id:
1. Creates an Argon2 context configured with the `password_secret` from the configuration.
2. Hashes the password with a randomly generated salt.
3. Returns the full hash string.

### `is_valid_password`
Verifies a plaintext password against a stored Argon2id hash:
1. Creates the Argon2 context with the `password_secret`.
2. Parses the stored hash string.
3. Verifies the password against the parsed hash.

### `create_argon_context`
Creates an `Argon2` instance using:
- The `password_secret` from config as the secret key.
- Default algorithm (Argon2id).
- Default version.
- Default params.

This means the password hash cannot be verified without the server's secret, adding an extra layer of protection against offline cracking.
