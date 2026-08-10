# Token Crypto (`auth/token_crypto.rs`)

## Summary

This module encrypts and decrypts user tokens at rest. It uses AES-256-GCM with a 32-byte key. The plaintext token is never stored in the database. The `user_token.token_secret` column stores a single encrypted blob: a 12-byte nonce followed by the ciphertext.

The key comes from the `token_encryption_secret` config value, a 64-character hexadecimal string. The config validates the key at startup.

## Imports

- `aes_gcm::aead::{Aead, KeyInit}`: AES-GCM traits.
- `aes_gcm::{Aes256Gcm, Nonce}`: The AES-256-GCM cipher and nonce type.
- `rand::rngs::OsRng`: Operating-system random number generator.
- `rand::RngCore`: Random byte filling.
- `thiserror::Error`: Error derive.

## Constant: `NONCE_LEN`

```rust
const NONCE_LEN: usize = 12;
```

Nonce length for AES-256-GCM (96 bits).

## Enum: `TokenCryptoError`

```rust
#[derive(Debug, Error)]
pub enum TokenCryptoError {
    #[error("AES-256-GCM decryption failed")]
    Decrypt,
    #[error("decrypted token is not valid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("stored token blob is too short to contain a nonce")]
    ShortBlob,
}
```

Errors from decrypting a stored user token.

- `Decrypt`: AES-256-GCM decryption failed (for example, a wrong key or corrupted ciphertext).
- `Utf8(FromUtf8Error)`: The decrypted plaintext is not valid UTF-8.
- `ShortBlob`: The stored blob is shorter than the 12-byte nonce.

## Function: `parse_key`

```rust
pub fn parse_key(hex_key: &str) -> Result<[u8; 32], String>
```

Parses a 64-character hex string into the 32-byte AES-256-GCM key.

**Algorithm:**
1. Decode the hex string with `hex::decode`.
2. If decoding fails, return `"token_encryption_secret must be a 64-character hexadecimal string"`.
3. Convert the bytes to a 32-byte array with `try_into`.
4. If the byte length is not 32, return `"token_encryption_secret must be exactly 32 bytes (64 hexadecimal characters)"`.

## Function: `encrypt`

```rust
pub fn encrypt(key: &[u8; 32], token: &str) -> (Vec<u8>, Vec<u8>)
```

Encrypts `token` with a fresh 12-byte nonce. Returns `(nonce, ciphertext)`.

**Algorithm:**
1. Create an `Aes256Gcm` cipher from `key`.
2. Generate a fresh 12-byte nonce with `OsRng.fill_bytes`.
3. Encrypt `token.as_bytes()` with the cipher and the nonce. AES-256-GCM encryption cannot fail; the code uses `expect`.
4. Return the nonce and the ciphertext.

## Function: `decrypt`

```rust
pub fn decrypt(key: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> Result<String, TokenCryptoError>
```

Decrypts `ciphertext` using the given `nonce`.

**Algorithm:**
1. Create an `Aes256Gcm` cipher from `key`.
2. Decrypt the ciphertext with the nonce.
3. On failure, return `TokenCryptoError::Decrypt`.
4. Convert the plaintext bytes to UTF-8 with `String::from_utf8`. On failure, return `TokenCryptoError::Utf8`.

## Function: `encrypt_blob`

```rust
pub fn encrypt_blob(key: &[u8; 32], token: &str) -> Vec<u8>
```

Encrypts `token` into a single stored blob: 12-byte nonce followed by ciphertext.

**Algorithm:**
1. Call `encrypt(key, token)` to get the nonce and the ciphertext.
2. Start the blob with the nonce.
3. Append the ciphertext to the blob.
4. Return the blob.

## Function: `decrypt_blob`

```rust
pub fn decrypt_blob(key: &[u8; 32], blob: &[u8]) -> Result<String, TokenCryptoError>
```

Decrypts a blob produced by `encrypt_blob`.

**Algorithm:**
1. Split the blob: the first `NONCE_LEN` bytes are the nonce, the rest is the ciphertext.
2. If the blob is shorter than `NONCE_LEN`, return `TokenCryptoError::ShortBlob`.
3. Call `decrypt(key, nonce, ciphertext)` and return the result.

## Key Source

- Config key: `token_encryption_secret` in `config.rs`. Type: `SmallString`. It must be a 64-character hexadecimal string (32 bytes).
- Default value: `"change"` (placeholder). The config rejects the placeholder, an empty value, and a value that `parse_key` cannot parse; startup fails with an error message.
- `Config::token_encryption_key()` calls `parse_key` on the config value and returns the 32-byte key. It uses `expect` because the config validated the key at startup.

## Stored Blob Layout

| Byte range | Content |
|------------|---------|
| 0–11 | Nonce (12 bytes, random per encryption) |
| 12–end | AES-256-GCM ciphertext |

## Usage

- `api/user_token.rs` `create`: encrypts the token with `encrypt_blob` and stores the result in `token_secret`. The hash of the token is stored in `token_hash`.
- `api/user_token.rs` `list` and `update`: call `decrypt_blob` to show the plaintext token to the owner only. On a decrypt failure, the code logs a warning and omits the token.
