# Authentication System

## Individual Files

- [header.md](header.md)
- [password.md](password.md)

## Purpose

The auth module provides client identity verification. It supports two authentication methods: Basic (username + password) and Token (username + token UUID). It also provides Argon2id password hashing.

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

### `authenticate_user`
The main entry point. Parses the Authorization header value:
1. Splits on the first space to get the auth type and credentials.
2. Routes to `basic_access_authentication` for "Basic" or `token_authentication` for "Token".
3. Returns `InvalidAuthType` for unknown types.

### Basic Access Authentication
1. Decodes the base64-encoded `username:password` pair.
2. Queries the database for the user's ID, rank, and password hash.
3. If the user does not exist, uses a dummy password hash to prevent timing attacks.
4. Verifies the password against the stored hash using Argon2id.
5. Returns the Client on success, `UsernamePasswordMismatch` on failure.

### Token Authentication
1. Decodes the base64-encoded `username:token` pair.
2. Parses the token as a UUID.
3. Joins `user_token` with `user` to get the user ID, rank, token enabled status, and expiration time.
4. Checks that the token is enabled and not expired.
5. Returns the Client on success, `UsernameTokenMismatch` or `ExpiredToken` on failure.

### Helper Functions
- `decode_credentials`: Base64 decodes the credentials string and splits on `:`.
- `basic_credentials_for`: Test helper that creates a "Basic" auth header value.
- `token_credentials_for`: Test helper that creates a "Token" auth header value.

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
