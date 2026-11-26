use aes_gcm::{
    Aes256Gcm, KeyInit,
    aead::{Aead, Payload},
};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::error::{AppError, AppResult};

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 12;

pub type MasterKey = Zeroizing<[u8; KEY_LEN]>;

pub trait ValueCrypto: Send + Sync {
    fn encrypt(&self, key: &[u8; KEY_LEN], plaintext: &[u8]) -> AppResult<(Vec<u8>, [u8; NONCE_LEN])>;
    fn decrypt(&self, key: &[u8; KEY_LEN], ciphertext: &[u8], nonce: &[u8; NONCE_LEN]) -> AppResult<Vec<u8>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Aes256GcmCrypto;

impl ValueCrypto for Aes256GcmCrypto {
    fn encrypt(&self, key: &[u8; KEY_LEN], plaintext: &[u8]) -> AppResult<(Vec<u8>, [u8; NONCE_LEN])> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| AppError::Crypto("invalid encryption key".into()))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, Payload { msg: plaintext, aad: b"" })
            .map_err(|_| AppError::Crypto("encryption failed".into()))?;
        Ok((ciphertext, nonce_bytes))
    }

    fn decrypt(&self, key: &[u8; KEY_LEN], ciphertext: &[u8], nonce: &[u8; NONCE_LEN]) -> AppResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| AppError::Crypto("invalid encryption key".into()))?;
        let nonce = aes_gcm::Nonce::from_slice(nonce);
        let plaintext = cipher
            .decrypt(nonce, Payload { msg: ciphertext, aad: b"" })
            .map_err(|_| AppError::Crypto("decryption failed".into()))?;
        Ok(plaintext)
    }
}

pub fn generate_master_key() -> MasterKey {
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    rand::rngs::OsRng.fill_bytes(key.as_mut());
    key
}

pub fn encode_key(key: &[u8; KEY_LEN]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(KEY_LEN * 2);
    for b in key {
        write!(&mut out, "{b:02x}").expect("hex write");
    }
    out
}

pub fn decode_key(hex: &str) -> AppResult<MasterKey> {
    if hex.len() != KEY_LEN * 2 {
        return Err(AppError::Keychain("invalid master key length".into()));
    }
    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        let hi = hex_nibble(chunk[0])?;
        let lo = hex_nibble(chunk[1])?;
        key[i] = (hi << 4) | lo;
    }
    Ok(key)
}

fn hex_nibble(b: u8) -> AppResult<u8> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(AppError::Keychain("invalid master key encoding".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_encrypt_decrypt() {
        let crypto = Aes256GcmCrypto;
        let key = generate_master_key();
        let plaintext = b"secret=value";
        let (ciphertext, nonce) = crypto.encrypt(&key, plaintext).unwrap();
        let decrypted = crypto.decrypt(&key, &ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn different_nonces_for_same_plaintext() {
        let crypto = Aes256GcmCrypto;
        let key = generate_master_key();
        let plaintext = b"same-value";
        let (c1, n1) = crypto.encrypt(&key, plaintext).unwrap();
        let (c2, n2) = crypto.encrypt(&key, plaintext).unwrap();
        assert_ne!(n1, n2);
        assert_ne!(c1, c2);
        assert_eq!(crypto.decrypt(&key, &c1, &n1).unwrap(), plaintext);
        assert_eq!(crypto.decrypt(&key, &c2, &n2).unwrap(), plaintext);
    }

    #[test]
    fn tamper_detection() {
        let crypto = Aes256GcmCrypto;
        let key = generate_master_key();
        let (mut ciphertext, nonce) = crypto.encrypt(&key, b"secret").unwrap();
        ciphertext[0] ^= 0x01;
        let err = crypto.decrypt(&key, &ciphertext, &nonce).unwrap_err();
        assert!(matches!(err, AppError::Crypto(_)));
    }

    #[test]
    fn wrong_key_fails() {
        let crypto = Aes256GcmCrypto;
        let key1 = generate_master_key();
        let key2 = generate_master_key();
        let (ciphertext, nonce) = crypto.encrypt(&key1, b"secret").unwrap();
        let err = crypto.decrypt(&key2, &ciphertext, &nonce).unwrap_err();
        assert!(matches!(err, AppError::Crypto(_)));
    }

    #[test]
    fn key_hex_round_trip() {
        let key = generate_master_key();
        let hex = encode_key(&key);
        let decoded = decode_key(&hex).unwrap();
        assert_eq!(key.as_ref(), decoded.as_ref());
    }

    #[test]
    fn decode_key_rejects_bad_input() {
        assert!(decode_key("short").is_err());
        assert!(decode_key(&"zz".repeat(32)).is_err());
    }
}
