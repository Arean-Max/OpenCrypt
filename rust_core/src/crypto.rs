use crate::error::{CryptError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::Engine;

pub const MAGIC: &[u8] = b"ORP\0";
pub const FORMAT_VERSION: u8 = 1;
pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const CHUNK_SIZE: usize = 1024 * 1024;
const TAG_LEN: usize = 16;
const PLAINTEXT_LEN_LEN: usize = 8;

pub const HEADER_LEN: usize = MAGIC.len() + 1 + PLAINTEXT_LEN_LEN + SALT_LEN + NONCE_LEN;

pub struct Header {
    pub plaintext_len: u64,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

pub fn generate_random_key() -> Result<String> {
    let mut key = [0u8; 32];
    getrandom::getrandom(&mut key).map_err(|_| CryptError::Memory)?;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&key);
    Ok(b64)
}
