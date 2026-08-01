# Password Hashing (`auth/password.rs`)

## Summary

This module handles password hashing and verification using Argon2id, a memory-hard key derivation function. It uses a secret key from the configuration for additional security. A constant dummy hash is used for constant-time comparison when verifying passwords for non-existent users.

## Imports

- `config::Config`: Application configuration.
- `argon2::password_hash::{Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}`: Argon2 password hashing API.
- `argon2::{Algorithm, Params, Version}`: Argon2 algorithm parameters.
- `argon2::{Argon2, Error as ArgonError}`: Argon2 context and error types.

## Constant: `DUMMY_PASSWORD_HASH`

```rust
pub const DUMMY_PASSWORD_HASH: &str =
    "$argon2id$v=19$m=19456,t=2,p=1$AAAAAAAAAAAAAAAAAAAAAA$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
```

A hardcoded Argon2id hash string used as a fallback when a username is not found. This prevents attackers from determining which usernames exist by measuring response time (timing side channel). The hash parameters are: Argon2id, version 19, 19456 KiB memory, 2 iterations, 1 degree of parallelism. The salt (`AAAA...`) and hash output are all-zero placeholders.

## Function: `hash_password(config, password, salt) -> Result<String, Error>`

```rust
pub fn hash_password(config: &Config, password: &str, salt: &SaltString) -> Result<String, Error>
```

Hashes a plaintext password using Argon2id.

**Algorithm:**
1. Call `create_argon_context(config)` to build an `Argon2` instance with the secret key.
2. Call `argon_context.hash_password(password.as_bytes(), salt)` to produce a password hash.
3. Return the hash as a string (`password_hash.to_string()`).

The salt is provided by the caller (generated with `SaltString::generate`).

## Function: `is_valid_password(config, password_hash, password) -> Result<(), Error>`

```rust
pub fn is_valid_password(config: &Config, password_hash: &str, password: &str) -> Result<(), Error>
```

Verifies that a plaintext password matches a stored hash.

**Algorithm:**
1. Call `create_argon_context(config)` to build an `Argon2` instance.
2. Parse the stored hash: `PasswordHash::new(password_hash)`.
3. Call `argon_context.verify_password(password.as_bytes(), &parsed_hash)`.
4. Return `Ok(())` on match, `Err` on mismatch or parse error.

## Function: `create_argon_context(config: &Config) -> Result<Argon2<'_>, ArgonError>`

Creates an `Argon2` context with the application secret.

**Algorithm:**
1. Call `Argon2::new_with_secret()` with:
   - `secret`: `config.password_secret.as_bytes()` (a secret key from configuration).
   - `algorithm`: `Algorithm::default()` (Argon2id).
   - `version`: `Version::default()` (V0x19).
   - `params`: `Params::default()` (standard memory/iteration/parallelism parameters).

Using `new_with_secret` instead of `new` adds a pepper — an additional secret key that makes pre-computed attacks infeasible even if the password hashes are leaked.

## Tests: `hash_password`

Creates a test config and verifies:
- `is_valid_password(test_config, TEST_HASH, TEST_PASSWORD)` returns `Ok`.
- `is_valid_password(test_config, TEST_HASH, "wrong_password")` returns `Err`.
