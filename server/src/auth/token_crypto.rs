use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use thiserror::Error;

/// Nonce length for AES-256-GCM (96 bits).
const NONCE_LEN: usize = 12;

/// Errors from decrypting a stored user token.
#[derive(Debug, Error)]
pub enum TokenCryptoError {
    #[error("AES-256-GCM decryption failed")]
    Decrypt,
    #[error("decrypted token is not valid UTF-8")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("stored token blob is too short to contain a nonce")]
    ShortBlob,
}

/// Parses a 64-character hex string into the 32-byte AES-256-GCM key.
pub fn parse_key(hex_key: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(hex_key).map_err(|_| {
        "token_encryption_secret must be a 64-character hexadecimal string".to_string()
    })?;
    bytes.try_into().map_err(|_| {
        "token_encryption_secret must be exactly 32 bytes (64 hexadecimal characters)".to_string()
    })
}

/// Encrypts `token` with a fresh 12-byte nonce.
/// Returns `(nonce, ciphertext)`.
pub fn encrypt(key: &[u8; 32], token: &str) -> (Vec<u8>, Vec<u8>) {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), token.as_bytes())
        .expect("AES-256-GCM encryption must not fail");
    (nonce.to_vec(), ciphertext)
}

/// Decrypts `ciphertext` using the given `nonce`.
pub fn decrypt(key: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> Result<String, TokenCryptoError> {
    let cipher = Aes256Gcm::new(key.into());
    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce), ciphertext)
        .map_err(|_| TokenCryptoError::Decrypt)?;
    Ok(String::from_utf8(plaintext)?)
}

/// Encrypts `token` into a single stored blob: 12-byte nonce followed by ciphertext.
pub fn encrypt_blob(key: &[u8; 32], token: &str) -> Vec<u8> {
    let (nonce, ciphertext) = encrypt(key, token);
    let mut blob = nonce;
    blob.extend_from_slice(&ciphertext);
    blob
}

/// Decrypts a blob produced by [`encrypt_blob`].
pub fn decrypt_blob(key: &[u8; 32], blob: &[u8]) -> Result<String, TokenCryptoError> {
    let (nonce, ciphertext) = blob
        .get(..NONCE_LEN)
        .zip(blob.get(NONCE_LEN..))
        .ok_or(TokenCryptoError::ShortBlob)?;
    decrypt(key, nonce, ciphertext)
}
